(() => {
  // create the table of contents
  document.addEventListener("DOMContentLoaded", () => {
    // find the main content
    const mainContent = document.querySelector("#content > main");
    const headings = mainContent.querySelectorAll("h1, h2, h3");

    // build up the table of contents
    const toc = [];
    for (const heading of headings) {
      const tagName = heading.tagName;

      if (tagName === "H1") {
        toc.push({ name: heading.innerText, id: heading.id, children: [] });
      } else {
        toc[toc.length - 1].children.push({
          name: heading.innerText,
          id: heading.id,
        });
      }
    }

    // create the HTML for this table of contents
    const tocWrapper = document.createElement("div");
    tocWrapper.id = "table-of-contents-wrapper";
    const tocNav = document.createElement("nav");
    tocNav.id = "table-of-contents";
    tocWrapper.appendChild(tocNav);
    const tocList = document.createElement("ol");
    for (const tocEntry of toc) {
      const entryLi = document.createElement("li");
      entryLi.appendChild(createLink(tocEntry));
      entryLi.classList.add("primary");

      if (tocEntry.children.length > 0) {
        const childList = document.createElement("ol");

        for (const childEntry of tocEntry.children) {
          const childLi = document.createElement("li");
          childLi.classList.add("secondary");
          childLi.appendChild(createLink(childEntry));
          childList.appendChild(childLi);
        }

        entryLi.appendChild(childList);
      }

      tocList.appendChild(entryLi);
    }

    tocNav.appendChild(tocList);

    document
      .querySelector("#content")
      .insertAdjacentElement("beforebegin", tocWrapper);

    markCurrentHeaderAsActive();
  });

  function createLink(entry) {
    const aTag = document.createElement("a");
    aTag.href = "#" + entry.id;
    aTag.innerText = entry.name;
    return aTag;
  }

  let shouldHandleScroll = true;

  // Add the .active class to the currently active table of contents node
  function markCurrentHeaderAsActive() {
    if (shouldHandleScroll) {
      shouldHandleScroll = false;
      requestAnimationFrame(() => (shouldHandleScroll = true));
    } else {
      return;
    }

    const tocNode = document.querySelector("#table-of-contents");
    if (!tocNode) {
      return;
    }

    const tocLinks = Array.from(tocNode.querySelectorAll("a"));
    tocLinks.forEach((tocLink) => tocLink.classList.remove("active"));

    const entries = Array.from(document.querySelectorAll("h1, h2, h3"));
    const scrollPos = window.scrollY;

    const entry = entries.find((entry) => entry.offsetTop > scrollPos);
    if (!entry) {
      return;
    }

    const entryLink = "#" + entry.id;
    tocLinks
      .find((tocLink) => tocLink.href.endsWith(entryLink))
      .classList.add("active");
  }

  window.addEventListener("scroll", markCurrentHeaderAsActive);
})();
