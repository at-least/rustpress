/* Scrollspy: outline active anchor + marker, and the navbar "top" state
   (transparent home navbar) driven by window.scrollY. */

var d = document;

export function init() {
  var navbar = d.getElementById("VPNavBar");
  var outlineContainer = d.querySelector(".VPDocAsideOutline");
  var outlineMarker = d.getElementById("VPOutlineMarker");
  var outlineLinks = outlineContainer
    ? Array.prototype.slice.call(outlineContainer.querySelectorAll("a.outline-link"))
    : [];

  function scrollTick() {
    if (navbar) {
      navbar.classList.toggle("top", window.scrollY <= 0);
    }
    if (!outlineLinks.length) return;
    var activeLink = null;
    outlineLinks.forEach(function (link) {
      var id = decodeURIComponent(link.hash.slice(1));
      var heading = d.getElementById(id);
      if (!heading) return;
      link.classList.remove("active");
      if (heading.getBoundingClientRect().top < 96) activeLink = link;
    });
    if (activeLink) {
      activeLink.classList.add("active");
      if (outlineMarker) {
        outlineMarker.style.opacity = "1";
        outlineMarker.style.top =
          activeLink.offsetTop + 19 - 9 + "px";
      }
    } else if (outlineMarker) {
      outlineMarker.style.opacity = "0";
    }
  }

  var ticking = false;
  window.addEventListener(
    "scroll",
    function () {
      if (!ticking) {
        ticking = true;
        requestAnimationFrame(function () {
          scrollTick();
          ticking = false;
        });
      }
    },
    { passive: true }
  );
  scrollTick();
}
