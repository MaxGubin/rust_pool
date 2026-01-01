
use core::pin::pin;
use log::{error, info, trace};
use rs_matter::{clusters, devices, Matter, MATTER_PORT};
use rs_matter::utils::storage::pooled::PooledBuffers;
use rs_matter::dm::{
    Async, AsyncHandler, AsyncMetadata, DataModel, Dataver, EmptyHandler, Endpoint, EpClMatcher,
    Node,
};


static MATTER: StaticCell<Matter> = StaticCell::new();
static BUFFERS: StaticCell<PooledBuffers<10, NoopRawMutex, IMBuffer>> = StaticCell::new();
static SUBSCRIPTIONS: StaticCell<DefaultSubscriptions> = StaticCell::new();


fn run() -> Result<(), Error> {
    info!(
        "Matter memory: Matter (BSS)={}B, IM Buffers (BSS)={}B, Subscriptions (BSS)={}B",
        core::mem::size_of::<Matter>(),
        core::mem::size_of::<PooledBuffers<10, NoopRawMutex, IMBuffer>>(),
        core::mem::size_of::<DefaultSubscriptions>()
    );

    let matter = MATTER.uninit().init_with(Matter::init(
        &TEST_DEV_DET,
        TEST_DEV_COMM,
        &TEST_DEV_ATT,
        rs_matter::utils::epoch::sys_epoch,
        rs_matter::utils::rand::sys_rand,
        MATTER_PORT,
    ));

    // Need to call this once
    matter.initialize_transport_buffers()?;

    // Create the transport buffers
    let buffers = BUFFERS.uninit().init_with(PooledBuffers::init(0));

    // Create the subscriptions
    let subscriptions = SUBSCRIPTIONS
        .uninit()
        .init_with(DefaultSubscriptions::init());

    // Our on-off cluster
    let on_off_handler = on_off::OnOffHandler::new_standalone(
        Dataver::new_rand(matter.rand()),
        1,
        TestOnOffDeviceLogic::new(true),
    );

    // Create the Data Model instance
    let dm = DataModel::new(
        matter,
        buffers,
        subscriptions,
        dm_handler(matter, &on_off_handler),
    );

    // Create a default responder capable of handling up to 3 subscriptions
    // All other subscription requests will be turned down with "resource exhausted"
    let responder = DefaultResponder::new(&dm);
    info!(
        "Responder memory: Responder (stack)={}B, Runner fut (stack)={}B",
        core::mem::size_of_val(&responder),
        core::mem::size_of_val(&responder.run::<4, 4>())
    );

    // Run the responder with up to 4 handlers (i.e. 4 exchanges can be handled simultaneously)
    // Clients trying to open more exchanges than the ones currently running will get "I'm busy, please try again later"
    let mut respond = pin!(responder.run::<4, 4>());

    // Run the background job of the data model
    let mut dm_job = pin!(dm.run());

    // Create, load and run the persister
    let socket = async_io::Async::<UdpSocket>::bind(MATTER_SOCKET_BIND_ADDR)?;

    info!(
        "Transport memory: Transport fut (stack)={}B, mDNS fut (stack)={}B",
        core::mem::size_of_val(&matter.run(&socket, &socket)),
        core::mem::size_of_val(&mdns::run_mdns(matter))
    );

    // Run the Matter and mDNS transports
    let mut mdns = pin!(mdns::run_mdns(matter));
    let mut transport = pin!(matter.run(&socket, &socket));

    // Create, load and run the persister
    let psm = PSM.uninit().init_with(Psm::init());
    let path = std::env::temp_dir().join("rs-matter");

    info!(
        "Persist memory: Persist (BSS)={}B, Persist fut (stack)={}B",
        core::mem::size_of::<Psm<4096>>(),
        core::mem::size_of_val(&psm.run(&path, matter, NO_NETWORKS))
    );

    psm.load(&path, matter, NO_NETWORKS)?;

    if !matter.is_commissioned() {
        // If the device is not commissioned yet, print the QR text and code to the console
        // and enable basic commissioning

        matter.print_standard_qr_text(DiscoveryCapabilities::IP)?;
        matter.print_standard_qr_code(QrTextType::Unicode, DiscoveryCapabilities::IP)?;

        matter.open_basic_comm_window(MAX_COMM_WINDOW_TIMEOUT_SECS)?;
    }

    let mut persist = pin!(psm.run(&path, matter, NO_NETWORKS));

    // Combine all async tasks in a single one
    let all = select4(
        &mut transport,
        &mut mdns,
        &mut persist,
        select(&mut respond, &mut dm_job).coalesce(),
    );

    // Run with a simple `block_on`. Any local executor would do.
    futures_lite::future::block_on(all.coalesce())
}
