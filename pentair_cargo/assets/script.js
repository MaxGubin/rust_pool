function toggleState(buttonId) {
  const button = document.getElementById(buttonId);
  const currentState = button.classList.contains('on') ? 'on' : 'off';

  // Check that the websocket is in a good shape.
  if (window.sharedWebSocket.readyState != WebSocket.OPEN) {
    console.error(`Websocket is not open in control [[${buttonId}]]`);
    return;
  }
  const newState = currentState === 'on' ? 'off' : 'on';
  window.sharedWebSocket.send(JSON.stringify({ control_name: buttonId, state: newState }));
}

function setupWebSocket() {
  const hostname = window.location.hostname;
  const port = window.location.port;
  const ws_url = `ws://${hostname}:${port}/ws`;
  console.log(`Connecting with Websocket [[${ws_url}]]`)
  const socket = new WebSocket(ws_url);
  socket.addEventListener('open', (event) => {
      console.log('Connected to the server');
      const connection_status = document.getElementById("connection-status");
      Array.from(connection_status.classList).forEach(
        cn=>{connection_status.classList.remove(cn);}  
      );
      connection_status.classList.add("connected");
      connection_status.innerHTML="Connected";

  });

  socket.addEventListener('message', (event) => {
    console.log('Message from server ', event.data);
    read_state(event.data);
  });

  socket.addEventListener('close', (event) => {
      console.log('Disconnected from the server');
      const connection_status = document.getElementById("connection-status");
      connection_status.classList.remove("connected");
      connection_status.classList.add("disconnected");
      connection_status.innerHTML="Disconnected";
  });

  socket.addEventListener('error', (event) => {
    console.error('WebSocket error');
  });
  return socket;
}


function read_state(state_data) {
    const response = JSON.parse(state_data);
    for (const [control_name, state] of response.switches) {
        const button = document.getElementById(control_name);
        button.classList.remove('on', 'off');
        button.classList.add(state? 'on': 'off');
    }
}

async function showLog() {
  const urlToFetch = '/log';
  try {
    const response = await fetch(urlToFetch);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const html = await response.text();
    const logDisplay = document.getElementById('logdiv');
    logDisplay.innerHTML = html; // Display the log content in the textarea
    logDisplay.style.display = "block";
  } catch (error) {
    console.error('Error fetching log:', error);
  }
}


function closeLog() {
    const logDisplay = document.getElementById('logdiv');
    logDisplay.style.display = "none";
}
