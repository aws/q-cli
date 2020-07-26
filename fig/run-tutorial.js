
// Remove spellcheck, autocomplete, autocaps, and autocorrect from doc
removeAnnoyingSpellcheck();

// Add fig formatting to document
addATouchOfFig();


// Apply formatting to <pre> node
function preFormatting(preNode) {

  var wrapperDiv = document.createElement('div');
  wrapperDiv.classList.add("fig_wrapper");


  preNode.parentNode.insertBefore(wrapperDiv, preNode);
  wrapperDiv.appendChild(preNode);

  // Set up pear button
  var pearButton = document.createElement('button');
  pearButton.classList.add("fig_pearButton");
  pearButton.classList.add("fig_prePearButton");
  pearButton.innerHTML = '▶';
preNode.parentNode.insertBefore(pearButton, preNode);

var copyButton = document.createElement('button');
copyButton.classList.add("fig_pearButton");
copyButton.classList.add("fig_prePearButton");
copyButton.innerHTML = 'copy';

    //preNode.parentNode.insertBefore(copyButton, preNode);

  // #### Events ####

  // Show pear on mouse enter
  wrapperDiv.addEventListener('mouseenter', function(e) {
    pearButton.classList.add("fig_buttonShow");
    copyButton.classList.add("fig_buttonShow");

  });

  // Hide pear on mouse leave
  wrapperDiv.addEventListener('mouseleave', function(e) {
    pearButton.classList.remove("fig_buttonShow");
    copyButton.classList.remove("fig_buttonShow");

  });

  // Make tag content editable
  preNode.setAttribute("contenteditable", true);

  // Add click event listener to pear
  pearButton.addEventListener('click', function(e) {
    e.preventDefault();
    e.stopPropagation();

    fig.run(preNode.innerText.trim())

  });
    
    copyButton.addEventListener('click', function(e) {
      e.preventDefault();
      e.stopPropagation();

      navigator.clipboard.writeText(preNode.innerText)

    });

  // Add event listener to copy code on click (but not highlight)
  // preNode.addEventListener('click', function (e) {
  //   e.preventDefault();
  //   e.stopPropagation();

  //   if (window.getSelection().toString() === "") {
  //     var deepLink = "fig://insert?cmd=" + preNode.innerText;
  //     console.log("Insert: " + deepLink);

  //     // This should just insert the code, NOT run it
  //     // window.location.href = deeplink
  //   }

  //   else {
  //     console.log("Highlight: " + window.getSelection().toString())
  //   }


  // });
}




// #### Apply formatting to <code> node

function codeFormatting(codeNode) {

  var wrapperSpan = document.createElement('span');
  wrapperSpan.classList.add("fig_wrapper");


  codeNode.parentNode.insertBefore(wrapperSpan, codeNode);
  wrapperSpan.appendChild(codeNode);

  // Set up pear button
  var pearButton = document.createElement('button');
  pearButton.classList.add("fig_pearButton");
  pearButton.classList.add("fig_inlinePearButton");
  pearButton.innerHTML = '▶';

  codeNode.parentNode.insertBefore(pearButton, codeNode);

  // #### Events ####

  // Show pear on mouse enter
  wrapperSpan.addEventListener('mouseenter', function(e) {
    pearButton.classList.add("fig_buttonShow");
  });

  // Hide pear on mouse leave
  wrapperSpan.addEventListener('mouseleave', function(e) {
    pearButton.classList.remove("fig_buttonShow");
  });

  codeNode.setAttribute("contenteditable", true);

  // Add click event listener to pear
  pearButton.addEventListener('click', function(e) {
    e.preventDefault();
    e.stopPropagation();

    fig.run(codeNode.innerText.trim());

  });

  // Add event listener to copy code on click (but not highlight)
  // codeNode.addEventListener('click', function (e) {
  //   e.preventDefault();
  //   e.stopPropagation();

  //   if (window.getSelection().toString() === "") {
  //     var deepLink = "fig://insert?cmd=" + codeNode.innerText;
  //     console.log("Insert: " + deepLink);

  //     // This should just insert the code, NOT run it
  //     // window.location.href = deeplink
  //   }

  //   else {
  //     console.log("Highlight: " + window.getSelection().toString())
  //   }

  // });
}



// #### Adds a 🍐 on hover for <code> and <pre> tags
function addATouchOfFig() {

  // Loop through <code> element
  // (and exclude code elements that have <pre> as a parent)
  var codes = document.querySelectorAll('code');
  codes.forEach(function(codeNode) {

    // Check if code is wrapped in <pre> or just <code>
    if (codeNode.parentNode.nodeName !== "PRE") {
      codeFormatting(codeNode);
    }
  });

  // Loop through <pre> element
  var pres = document.querySelectorAll('pre');
  pres.forEach(function(preNode) {
       if (!preNode.getAttribute('figdisabled')) {
          preFormatting(preNode);
       }
  });
}


function removeAnnoyingSpellcheck() {
  var allInputs = document.querySelectorAll('input');
  allInputs.forEach(function(myInput) {

    myInput.setAttribute("spellcheck", false);
    myInput.setAttribute("autocomplete", "off");
    myInput.setAttribute("autocapitalize", "off");
    myInput.setAttribute("autocorrect", "off");

  });
}

