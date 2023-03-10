function request(path, callback) {
  let xmlHttp = new XMLHttpRequest();
  xmlHttp.onreadystatechange = function() {
    if (xmlHttp.readyState == 4 && xmlHttp.status == 200)
        callback(xmlHttp.responseText);
  }
  xmlHttp.open("GET", path, true);
  xmlHttp.send(null);
}

function list_doors() {
  console.log("Listing doors");
  request("/api/v1/doors", function(text) {
      const config = JSON.parse(text);
  });
}

let connected = false;
let prev_connected = false;

setInterval(function() {
  request("/api/v1/poll", function(text) {
      const poll = JSON.parse(text);
      connected = poll.available;

      if (connected && !prev_connected) {
          let color = "#66ff66";
          document.body.style.backgroundColor = color;
          list_doors();
      } else if (!connected && prev_connected) {
          let color = "#ff6666";
          document.body.style.backgroundColor = color;
      }

      prev_connected = connected;
  });
}, 1000);

