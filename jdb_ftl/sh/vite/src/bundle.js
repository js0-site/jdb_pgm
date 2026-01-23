import "./main.styl";
import mermaid from "mermaid";
import katex from "katex";

mermaid.initialize({
  startOnLoad: false,
  theme: "default",
  securityLevel: "loose",
});

window.mermaid = mermaid;
window.katex = katex;

// Function to safely run mermaid
const initMermaid = async () => {
  try {
    const elements = document.querySelectorAll(".mermaid");
    if (elements.length > 0) {
      console.log(`Found ${elements.length} mermaid elements, rendering...`);
      await mermaid.run({
        nodes: elements,
      });
      console.log("Mermaid rendering complete");
    }
  } catch (e) {
    console.error("Mermaid run failed:", e);
  } finally {
    window.mermaidDone = true;
  }
};

// Execute immediately and on DOMContentLoaded
if (typeof window !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initMermaid);
  } else {
    initMermaid();
  }

  // Also try after a short delay to be safe
  setTimeout(initMermaid, 500);
}
