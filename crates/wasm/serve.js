const fs = require("fs");
const http = require("http");
const path = require("path");

const port = 8080;

http
  .createServer((req, res) => {
    let encoding = "utf8";
    let url = req.url;
    if (url.endsWith(".wasm")) {
      encoding = null;
    }
    if (url == "/") {
      url = "/index.html";
    }
    let filePath = path.join(__dirname, "/www", url);

    fs.readFile(filePath, { encoding }, function (err, fileData) {
      if (err) {
        console.error(err);
        res.writeHead(404);
        res.end(JSON.stringify(err));
        return;
      }
      let contentType = "text";
      if (url.endsWith(".js")) {
        contentType = "text/javascript";
      } else if (url.endsWith(".html")) {
        contentType = "text/html";
      } else if (url.endsWith(".css")) {
        contentType = "text/css";
      } else if (url.endsWith(".wasm")) {
        contentType = "application/wasm";
      }
      res.setHeader("Content-Type", contentType);
      res.writeHead(200);
      res.end(fileData);
    });
  })
  .listen(port, "localhost");

console.log(`UI available at http://localhost:${port}`);
