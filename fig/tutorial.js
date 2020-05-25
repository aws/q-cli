//
function preFormatting(preNode) {

 var wrapperDiv = document.createElement('div');
 wrapperDiv.classList.add("wrapper");


 preNode.parentNode.insertBefore(wrapperDiv, preNode);
 wrapperDiv.appendChild(preNode);

 // Set up pear button
 var pearButton = document.createElement('button');
 pearButton.classList.add("pearButton");
 pearButton.classList.add("prePearButton");
 pearButton.innerHTML = '🍐';

 preNode.parentNode.insertBefore(pearButton, preNode);

 // #### Events ####

 // Show pear on mouse enter
 wrapperDiv.addEventListener('mouseenter', function (e) {
   pearButton.classList.add("buttonShow");
 });

 // Hide pear on mouse leave
 wrapperDiv.addEventListener('mouseleave', function (e) {
   pearButton.classList.remove("buttonShow");
 });

 // Add click event listener to pear
 pearButton.addEventListener('click', function (e) {
   //e.preventDefault();
   //e.stopPropagation();

   var deepLink = "fig://insert?cmd=" + preNode.innerText;
   console.log("Pear: " + deepLink);

   window.webkit.messageHandlers.executeHandler.postMessage(preNode.innerText)
   // This should insert and run the code
   //window.location.href = deeplink
 });

 // Add event listener to copy code on click (but not highlight)
 preNode.addEventListener('click', function (e) {
   //e.preventDefault();
   //e.stopPropagation();

   if (window.getSelection().toString() === "") {
     var deepLink = "fig://insert?cmd=" + preNode.innerText;
     console.log("Insert: " + deepLink);

     // This should just insert the code, NOT run it
   window.webkit.messageHandlers.insertHandler.postMessage(preNode.innerText)

     //window.location.href = deeplink
   }

   else {
     console.log("Highlight: " + window.getSelection().toString())
   }


 });
}




// #### Apply formatting to <code> node

function codeFormatting(codeNode) {

 var wrapperSpan = document.createElement('span');
 wrapperSpan.classList.add("wrapper");


 codeNode.parentNode.insertBefore(wrapperSpan, codeNode);
 wrapperSpan.appendChild(codeNode);

 // Set up pear button
 var pearButton = document.createElement('button');
 pearButton.classList.add("pearButton");
 pearButton.classList.add("inlinePearButton");
 pearButton.innerHTML = '🍐';

 codeNode.parentNode.insertBefore(pearButton, codeNode);

 // #### Events ####

 // Show pear on mouse enter
 wrapperSpan.addEventListener('mouseenter', function (e) {
   pearButton.classList.add("buttonShow");
 });

 // Hide pear on mouse leave
 wrapperSpan.addEventListener('mouseleave', function (e) {
   pearButton.classList.remove("buttonShow");
 });

 // Add click event listener to pear
 pearButton.addEventListener('click', function (e) {
   //e.preventDefault();
   //e.stopPropagation();

   var deepLink = "fig://insert?cmd=" + encodeURI(codeNode.innerText);
   console.log("Pear: " + deepLink);
   window.webkit.messageHandlers.executeHandler.postMessage(codeNode.innerText)


   // This should insert and run the code
   //window.location.href = deeplink
 });

 // Add event listener to copy code on click (but not highlight)
 codeNode.addEventListener('click', function (e) {
   //e.preventDefault();
   //e.stopPropagation();

   if (window.getSelection().toString() === "") {
     var deepLink = "fig://insert?cmd=" + codeNode.innerText;
     console.log("Insert: " + deepLink);

     // This should just insert the code, NOT run it
     //window.location.href = deeplink
       window.webkit.messageHandlers.insertHandler.postMessage(codeNode.innerText)

   }

   else {
     console.log("Highlight: " + window.getSelection().toString())
   }

 });
}



// #### Adds a 🍐on hover for <code> and <pre> tags
function addATouchOfFig() {

 // Loop through <code> element
 // (and exclude code elements that have <pre> as a parent)
 var codes = document.querySelectorAll('code');
 codes.forEach(function (codeNode) {

   // Check if code is wrapped in <pre> or just <code>
   if (codeNode.parentNode.nodeName !== "PRE") {
     codeFormatting(codeNode);
   }
 });

 // Loop through <pre> element
 var pres = document.querySelectorAll('pre');
 pres.forEach(function (preNode) {
   preFormatting(preNode);
 });

 console.log("finished adding fig touchups");
}
addATouchOfFig()
