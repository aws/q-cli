window.addEventListener("load", () => {
  const $head = document.querySelector("head");
  const $body = document.querySelector("body");
  const script = $body.innerHTML.split("\n");
  const content = script.reduce(
    (content, line) => {
      if (line.slice(0, 2) === "##") {
        content.head.push(line.slice(2));
      } else {
        content.body.push(line);
      }
      return content;
    },
    { head: [], body: [] }
  );

  let rawCode = content.body.join("\n");
  rawCode = rawCode.replace(/&gt;/g, ">");
  rawCode = rawCode.replace(/&amp;/g, "&");

  const code = Prism.highlight(
    rawCode,
    Prism.languages.bash,
    "bash"
  );

  $body.innerHTML =
    "<pre class='language-bash'><code class='language-bash'>" +
    code +
    "</code></pre>";
  $head.innerHTML =
    "<title>Fig CLI Installer</title>" + content.head.join("\n");
});
