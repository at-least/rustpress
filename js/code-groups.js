/* Code groups: builds VitePress-style radio tabs from the `name=` fence
   annotation. */

var d = document;

export function init() {
  d.querySelectorAll(".vp-code-group").forEach(function (group, gi) {
    var blocks = group.querySelectorAll(".blocks > pre");
    if (blocks.length < 2) return;
    var allNamed = Array.prototype.every.call(blocks, function (pre) {
      return pre.querySelector("code[data-name]");
    });
    if (!allNamed) return;
    var tabs = d.createElement("div");
    tabs.className = "tabs";
    var groupName = "vp-tab-" + gi;
    blocks.forEach(function (pre, i) {
      var code = pre.querySelector("code[data-name]");
      var name = code ? code.getAttribute("data-name") : "";
      if (!name) return;
      var input = d.createElement("input");
      input.type = "radio";
      input.name = groupName;
      input.id = groupName + "-" + i;
      if (i === 0) input.checked = true;
      var label = d.createElement("label");
      label.htmlFor = input.id;
      label.textContent = name;
      input.addEventListener("change", function () {
        blocks.forEach(function (b) {
          b.classList.remove("active");
        });
        pre.classList.add("active");
      });
      tabs.appendChild(input);
      tabs.appendChild(label);
    });
    group.prepend(tabs);
    blocks[0].classList.add("active");
  });
}
