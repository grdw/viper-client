function request(path, token, callback) {
  let xmlHttp = new XMLHttpRequest();
  xmlHttp.onreadystatechange = function() {
    if (xmlHttp.readyState == 4 && xmlHttp.status == 200)
        callback(xmlHttp.responseText);
  }
  xmlHttp.open("GET", path, true);
  xmlHttp.send(null);
}

let connected = false;
let prev_connected = false;

setInterval(function() {
  request("/api/v1/poll", "", function(text) {
      let color = "#ffffff";
      const poll = JSON.parse(text);
      connected = poll.available;

      if (connected && !prev_connected) {
          color = "#66ff66";
      } else if (!connected && prev_connected) {
          color = "#ff6666";
      }

      document.body.style.backgroundColor = color;

      prev_connected = connected;
  });
}, 1000);

