/* vitezola — vanilla JS port of the VitePress default theme interactions.
   Entry point: each feature lives in its own module and exports init();
   they are called here in the same top-to-bottom order the original
   single-file script executed, so listener registration order is
   unchanged. */

import { init as initAppearance } from "./appearance.js";
import { init as initNavScreen, close as closeNavScreen } from "./nav-screen.js";
import { init as initSidebar, close as closeSidebar } from "./sidebar.js";
import { init as initFlyouts } from "./flyouts.js";
import { init as initNavScreenAccordions } from "./nav-screen-accordions.js";
import { init as initCodeGroups } from "./code-groups.js";
import { init as initCodeCopy } from "./code-copy.js";
import { init as initOutlineDropdown } from "./outline-dropdown.js";
import { init as initScrollspy } from "./scrollspy.js";
import { init as initSearch } from "./search.js";

initAppearance();
initNavScreen();
initSidebar();

/* The shared backdrop closes both overlays it can appear behind. */
var backdrop = document.getElementById("VPBackdrop");
if (backdrop) {
  backdrop.addEventListener("click", function () {
    closeNavScreen();
    closeSidebar();
  });
}

initFlyouts();
initNavScreenAccordions();
initCodeGroups();
initCodeCopy();
initOutlineDropdown();
initScrollspy();
initSearch();
