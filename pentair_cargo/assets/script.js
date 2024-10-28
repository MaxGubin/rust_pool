function toggleState(buttonId) {
    const button = document.getElementById(buttonId);
    const currentState = button.classList.contains('on') ? 'on' : 'off';

    const newState = currentState === 'on' ? 'off' : 'on';

    // Send POST request with button id and state
    const xhr = new XMLHttpRequest();
    xhr.open("POST", "/control", true);
    xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8");

    xhr.onload = function() {
        if (xhr.status >= 200 && xhr.status < 300) {
            console.log(`Response: ${xhr.responseText}`);
        } else {
            console.error(`Error: ${xhr.statusText}`);
        }
    };

    xhr.onerror = function() {
        console.error('Request failed');
    };

    xhr.send(JSON.stringify({ control_name: buttonId, state: newState }));
}

function setupWebSocket() {
  const hostname = window.location.hostname;
  const port = window.location.port;
  const ws_url = `ws://${hostname}:${port}/ws`;
  console.log(`Connecting with Websocket [[${ws_url}]]`)
  const socket = new WebSocket(ws_url);
  socket.addEventListener('open', (event) => {
      console.log('Connected to the server');
  });

  socket.addEventListener('message', (event) => {
      console.log('Message from server ', event.data);
  });

  socket.addEventListener('close', (event) => {
      console.log('Disconnected from the server');
  });
}
