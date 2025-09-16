function request(path, token, callback) {
  let xmlHttp = new XMLHttpRequest();
  xmlHttp.onreadystatechange = function() {
    if (xmlHttp.readyState == 4 && xmlHttp.status == 200)
        callback(xmlHttp.responseText);
  }
  xmlHttp.open("GET", path, true);
  xmlHttp.send(null);
}

function list_doors(token) {
  console.log("Listing doors");
  request("/api/v1/doors", token, function(text) {
      const config = JSON.parse(text);
  });
}

function get_sign_up_token() {
  let token = "";
  request("/api/v1/token", "", function(text) {
     const tokenResponse = JSON.parse(text);
     token = tokenResponse["user-token"];
  });
  return token;
}

let connected = false;
let prev_connected = false;

setInterval(function() {
  request("/api/v1/poll", "", function(text) {
      const poll = JSON.parse(text);
      connected = poll.available;

      if (connected && !prev_connected) {
          let color = "#66ff66";
          document.body.style.backgroundColor = color;
          let token = get_sign_up_token();
          console.log("Signed up with token", token);
          list_doors(token);
      } else if (!connected && prev_connected) {
          let color = "#ff6666";
          document.body.style.backgroundColor = color;
      }

      prev_connected = connected;
  });
}, 1000);

