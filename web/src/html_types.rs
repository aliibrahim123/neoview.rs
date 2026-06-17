//! items used in [`chunk`](crate::chunk!) macro intellisense.
//!
//! Requires the `html-types` feature.
#![allow(nonstandard_style, unused)]
/// HTML tags descriptions
pub mod html_tags {
	/// The [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/a) HTML element (or anchor element), with its href attribute, creates a hyperlink to web pages, files, email addresses, locations in the same page, or anything else a URL can address.
	pub struct a;
	/// The [`<abbr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/abbr) HTML element represents an abbreviation or acronym; the optional title attribute can provide an expansion or description for the abbreviation.
	pub struct abbr;
	/// The [`<address>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/address) HTML element indicates that the enclosed HTML provides contact information for a person or people, or for an organization.
	pub struct address;
	/// The [`<area>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/area) HTML element defines an area inside an image map that has predefined clickable areas.
	pub struct area;
	/// The [`<article>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/article) HTML element represents a self-contained composition in a document, page, application, or site, which is intended to be independently distributable or reusable (e.g., in syndication).
	pub struct article;
	/// The [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/aside) HTML element represents a portion of a document whose content is only indirectly related to the document's main content.
	pub struct aside;
	/// The [`<audio>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/audio) HTML element is used to embed sound content in documents.
	pub struct audio;
	/// The [`<b>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/b) HTML element is used to draw the reader's attention to the element's contents, which are not otherwise granted special importance.
	pub struct b;
	/// The [`<base>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/base) HTML element specifies the base URL to use for all relative URLs in a document.
	pub struct base;
	/// The [`<bdi>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/bdi) HTML element tells the browser's bidirectional algorithm to treat the text it contains in isolation from its surrounding text.
	pub struct bdi;
	/// The [`<bdo>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/bdo) HTML element overrides the current directionality of text, so that the text within is rendered in a different direction.
	pub struct bdo;
	/// The [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/blockquote) HTML element indicates that the enclosed text is an extended quotation.
	pub struct blockquote;
	/// The [`<body>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/body) HTML element represents the content of an HTML document.
	pub struct body;
	/// The [`<br>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/br) HTML element produces a line break in text (carriage-return).
	pub struct br;
	/// The [`<button>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/button) HTML element represents a clickable button, used to submit forms or anywhere in a document for accessible, standard button functionality.
	pub struct button;
	/// Use the HTML [`<canvas>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/canvas) element with either the canvas scripting API or the WebGL API to draw graphics and animations.
	pub struct canvas;
	/// The [`<caption>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/caption) HTML element specifies the caption (or title) of a table.
	pub struct caption;
	/// The [`<cite>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/cite) HTML element is used to describe a reference to a cited creative work, and must include the title of that work.
	pub struct cite;
	/// The [`<code>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/code) HTML element displays its contents styled in a fashion intended to indicate that the text is a short fragment of computer code.
	pub struct code;
	/// The [`<col>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/col) HTML element defines a column within a table and is used for defining common semantics on all common cells.
	pub struct col;
	/// The [`<colgroup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/colgroup) HTML element defines a group of columns within a table.
	pub struct colgroup;
	/// The [`<data>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/data) HTML element links a given piece of content with a machine-readable translation.
	pub struct data;
	/// The [`<datalist>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/datalist) HTML element contains a set of option elements that represent the permissible or recommended options available to choose from within other controls.
	pub struct datalist;
	/// The [`<dd>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dd) HTML element provides the description, definition, or value for the preceding term (dt) in a description list (dl).
	pub struct dd;
	/// The [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del) HTML element represents a range of text that has been deleted from a document.
	pub struct del;
	/// The [`<details>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/details) HTML element creates a disclosure widget in which information is visible only when the widget is toggled into an "open" state.
	pub struct details;
	/// The [`<dfn>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dfn) HTML element is used to indicate the term being defined within the context of a definition phrase or sentence.
	pub struct dfn;
	/// The [`<dialog>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dialog) HTML element represents a dialog box or other interactive component, such as a dismissible alert, inspector, or subwindow.
	pub struct dialog;
	/// The [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/div) HTML element is the generic container for flow content.
	pub struct div;
	/// The [`<dl>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dl) HTML element represents a description list.
	pub struct dl;
	/// The [`<dt>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dt) HTML element specifies a term in a description or definition list, and as such must be used inside a dl element.
	pub struct dt;
	/// The [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/em) HTML element marks text that has stress emphasis.
	pub struct em;
	/// The [`<embed>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/embed) HTML element embeds external content at the specified point in the document.
	pub struct embed;
	/// The [`<fieldset>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/fieldset) HTML element is used to group several controls as well as labels (label) within a web form.
	pub struct fieldset;
	/// The [`<figcaption>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/figcaption) HTML element represents a caption or legend describing the rest of the contents of its parent figure element.
	pub struct figcaption;
	/// The [`<figure>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/figure) HTML element represents self-contained content, potentially with an optional caption, which is specified using the figcaption element.
	pub struct figure;
	/// The [`<footer>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/footer) HTML element represents a footer for its nearest sectioning content or sectioning root element.
	pub struct footer;
	/// The [`<form>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/form) HTML element represents a document section containing interactive controls for submitting information.
	pub struct form;
	/// The [`<h1>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/Heading_Elements) represents the first level of heading.
	pub struct h1;
	/// The [`<h2>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/Heading_Elements) represents the second level of heading.
	pub struct h2;
	/// The [`<h3>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/Heading_Elements) represents the third level of heading.
	pub struct h3;
	/// The [`<h4>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/Heading_Elements) represents the forth level of heading.
	pub struct h4;
	/// The [`<h5>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/Heading_Elements) represents the fifth level of heading.
	pub struct h5;
	/// The [`<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/Heading_Elements) represents the sixth level of heading.
	pub struct h6;
	/// The [`<head>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/head) HTML element contains machine-readable information (metadata) about the document, like its title, scripts, and style sheets.
	pub struct head;
	/// The [`<header>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/header) HTML element represents introductory content, typically a group of introductory or navigational aids.
	pub struct header;
	/// The [`<hgroup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/hgroup) HTML element represents a heading and related content.
	pub struct hgroup;
	/// The [`<hr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/hr) HTML element represents a thematic break between paragraph-level elements: for example, a change of scene in a story, or a shift of topic within a section.
	pub struct hr;
	/// The [`<html>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/html) HTML element represents the root (top-level element) of an HTML document, so it is also referred to as the root element.
	pub struct html;
	/// The [`<i>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/i) HTML element represents a range of text that is set off from the normal text for some reason, such as idiomatic text, technical terms, taxonomical designations, among others.
	pub struct i;
	/// The [`<iframe>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe) HTML element represents a nested browsing context, embedding another HTML page into the current one.
	pub struct iframe;
	/// The [`<img>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img) HTML element embeds an image into the document.
	pub struct img;
	/// The [`<input>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input) HTML element is used to create interactive controls for web-based forms in order to accept data from the user; a wide variety of types of input data and control widgets are available, depending on the device and user agent.
	pub struct input;
	/// The [`<ins>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/ins) HTML element represents a range of text that has been added to a document.
	pub struct ins;
	/// The [`<kbd>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/kbd) HTML element represents a span of inline text denoting textual user input from a keyboard, voice input, or any other text entry device.
	pub struct kbd;
	/// The [`<label>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/label) HTML element represents a caption for an item in a user interface.
	pub struct label;
	/// The [`<legend>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/legend) HTML element represents a caption for the content of its parent fieldset.
	pub struct legend;
	/// The [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/li) HTML element is used to represent an item in a list.
	pub struct li;
	/// The [`<link>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/link) HTML element specifies relationships between the current document and an external resource.
	pub struct link;
	/// The [`<main>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/main) HTML element represents the dominant content of the body of a document.
	pub struct main;
	/// The [`<map>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/map) HTML element is used with area elements to define an image map (a clickable link area).
	pub struct map;
	/// The [`<mark>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/mark) HTML element represents text which is marked or highlighted for reference or notation purposes, due to the marked passage's relevance or importance in the enclosing context.
	pub struct mark;
	/// The [`<math>`](https://developer.mozilla.org/en-US/docs/Web/MathML/Reference/Element/math) MathML element is the top-level MathML element, used to write a single mathematical formula.
	pub struct math;
	/// The [`<menu>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/menu) HTML element is a semantic alternative to ul.
	pub struct menu;
	/// The [`<meta>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/meta) HTML element represents Metadata that cannot be represented by other HTML meta-related elements, like base, link, script, style or title.
	pub struct meta;
	/// The [`<meter>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/meter) HTML element represents either a scalar value within a known range or a fractional value.
	pub struct meter;
	/// The [`<nav>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/nav) HTML element represents a section of a page whose purpose is to provide navigation links, either within the current document or to other documents.
	pub struct nav;
	/// The [`<noscript>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/noscript) HTML element defines a section of HTML to be inserted if a script type on the page is unsupported or if scripting is currently turned off in the browser.
	pub struct noscript;
	/// The [`<object>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/object) HTML element represents an external resource, which can be treated as an image, a nested browsing context, or a resource to be handled by a plugin.
	pub struct object;
	/// The [`<ol>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/ol) HTML element represents an ordered list of items — typically rendered as a numbered list.
	pub struct ol;
	/// The [`<optgroup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/optgroup) HTML element creates a grouping of options within a select element.
	pub struct optgroup;
	/// The [`<option>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/option) HTML element is used to define an item contained in a `<select>`, an` <optgroup>`, or a `<datalist>` element.
	pub struct option;
	/// The [`<output>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/output) HTML element is a container element into which a site or app can inject the results of a calculation or the outcome of a user action.
	pub struct output;
	/// The [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/p) HTML element represents a paragraph.
	pub struct p;
	/// The [`<picture>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/picture) HTML element contains zero or more source elements and one img element to offer alternative versions of an image for different display/device scenarios.
	pub struct picture;
	/// The [`<portal>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/portal) HTML element enables the embedding of another HTML page into the current one for the purposes of allowing smoother navigation into new pages.
	pub struct portal;
	/// The [`<pre>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/pre) HTML element represents preformatted text which is to be presented exactly as written in the HTML file.
	pub struct pre;
	/// The [`<progress>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/progress) HTML element displays an indicator showing the completion progress of a task, typically displayed as a progress bar.
	pub struct progress;
	/// The [`<q>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/q) HTML element indicates that the enclosed text is a short inline quotation.
	pub struct q;
	/// The [`<rp>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/rp) HTML element is used to provide fall-back parentheses for browsers that do not support display of ruby annotations using the ruby element.
	pub struct rp;
	/// The [`<rt>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/rt) HTML element specifies the ruby text component of a ruby annotation, which is used to provide pronunciation, translation, or transliteration information for East Asian typography.
	pub struct rt;
	/// The [`<ruby>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/ruby) HTML element represents small annotations that are rendered above, below, or next to base text, usually used for showing the pronunciation of East Asian characters.
	pub struct ruby;
	/// The [`<s>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/s) HTML element renders text with a strikethrough, or a line through it.
	pub struct s;
	/// The [`<samp>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/samp) HTML element is used to enclose inline text which represents sample (or quoted) output from a computer program.
	pub struct samp;
	/// The [`<script>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/script) HTML element is used to embed executable code or data; this is typically used to embed or refer to JavaScript code.
	pub struct script;
	/// The [`<search>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/search) HTML element is a container representing the parts of the document or application with form controls or other content related to performing a search or filtering operation.
	pub struct search;
	/// The [`<section>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/section) HTML element represents a generic standalone section of a document, which doesn't have a more specific semantic element to represent it.
	pub struct section;
	/// The [`<select>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/select) HTML element represents a control that provides a menu of options:.
	pub struct select;
	/// The [`<slot>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/slot) HTML element—part of the Web Components technology suite—is a placeholder inside a web component that you can fill with your own markup, which lets you create separate DOM trees and present them together.
	pub struct slot;
	/// The [`<small>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/small) HTML element represents side-comments and small print, like copyright and legal text, independent of its styled presentation.
	pub struct small;
	/// The [`<source>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/source) HTML element specifies multiple media resources for the picture, the audio element, or the video element.
	pub struct source;
	/// The [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/span) HTML element is a generic inline container for phrasing content, which does not inherently represent anything.
	pub struct span;
	/// The [`<strong>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/strong) HTML element indicates that its contents have strong importance, seriousness, or urgency.
	pub struct strong;
	/// The [`<style>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/style) HTML element contains style information for a document, or part of a document.
	pub struct style;
	/// The [`<sub>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/sub) HTML element specifies inline text which should be displayed as subscript for solely typographical reasons.
	pub struct sub;
	/// The [`<summary>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/summary) HTML element specifies a summary, caption, or legend for a details element's disclosure box.
	pub struct summary;
	/// The [`<sup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/sup) HTML element specifies inline text which is to be displayed as superscript for solely typographical reasons.
	pub struct sup;
	/// The [`<svg>`](https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element/svg) SVG element is a container defining a new coordinate system and viewport.
	pub struct svg;
	/// The [`<table>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/table) HTML element represents tabular data — that is, information presented in a two-dimensional table comprised of rows and columns of cells containing data.
	pub struct table;
	/// The [`<tbody>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/tbody) HTML element encapsulates a set of table rows (tr elements), indicating that they comprise the body of the table (table).
	pub struct tbody;
	/// The [`<td>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/td) HTML element defines a cell of a table that contains data.
	pub struct td;
	/// The [`<template>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/template) HTML element is a mechanism for holding HTML that is not to be rendered immediately when a page is loaded but may be instantiated subsequently during runtime using JavaScript.
	pub struct template;
	/// The [`<textarea>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/textarea) HTML element represents a multi-line plain-text editing control, useful when you want to allow users to enter a sizeable amount of free-form text, for example a comment on a review or feedback form.
	pub struct textarea;
	/// The [`<tfoot>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/tfoot) HTML element defines a set of rows summarizing the columns of the table.
	pub struct tfoot;
	/// The [`<th>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/th) HTML element defines a cell as header of a group of table cells.
	pub struct th;
	/// The [`<thead>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/thead) HTML element defines a set of rows defining the head of the columns of the table.
	pub struct thead;
	/// The [`<time>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/time) HTML element represents a specific period in time.
	pub struct time;
	/// The [`<title>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/title) HTML element defines the document's title that is shown in a Browser's title bar or a page's tab.
	pub struct title;
	/// The [`<tr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/tr) HTML element defines a row of cells in a table.
	pub struct tr;
	/// The [`<track>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/track) HTML element is used as a child of the media elements, audio and video.
	pub struct track;
	/// The [`<u>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/u) HTML element represents a span of inline text which should be rendered in a way that indicates that it has a non-textual annotation.
	pub struct u;
	/// The [`<ul>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/ul) HTML element represents an unordered list of items, typically rendered as a bulleted list.
	pub struct ul;
	/// The [`<var>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/var) HTML element represents the name of a variable in a mathematical expression or a programming context.
	pub struct var;
	/// The [`<video>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/video) HTML element embeds a media player which supports video playback into the document.
	pub struct video;
	/// The [`<wbr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/wbr) HTML element represents a word break opportunity—a position within text where the browser may optionally break a line, though its line-breaking rules would not otherwise create a break at that location.
	pub struct wbr;
}

/// HTML attributes descriptions
pub mod html_attrs {
	/// The `abbr` attribute specifies an abbreviated form of the element's content.
	pub static abbr: () = ();
	/// The `accept` attribute specifies a list of types the server accepts, typically a file type.
	pub static accept: () = ();
	/// The `accept-charset` attribute specifies the character encodings that are to be used for the form submission.
	pub static accept_charset: () = ();
	/// The [`accesskey`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/accesskey) attribute specifies a shortcut key to activate or focus an element.
	pub static accesskey: () = ();
	/// The `action` attribute defines the URL to which the form data will be sent.
	pub static action: () = ();
	/// The `align` attribute specifies the alignment of an element.
	pub static align: () = ();
	/// The `allow` attribute defines a feature policy for the content in an iframe.
	pub static allow: () = ();
	/// The `allowfullscreen` attribute allows the iframe to be displayed in fullscreen mode.
	pub static allowfullscreen: () = ();
	/// The `allowpaymentrequest` attribute allows a cross-origin iframe to invoke the Payment Request API.
	pub static allowpaymentrequest: () = ();
	/// The `alt` attribute provides alternative text for an image, if the image cannot be displayed.
	pub static alt: () = ();
	/// The [`aria-activedescendant`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-activedescendant) attribute identifies the currently active element when DOM focus is on a composite widget, textbox, group, or application.
	pub static aria_activedescendant: () = ();
	/// The [`aria-atomic`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-atomic) attribute indicates whether assistive technologies will present all, or only parts of, the changed region based on the change notifications defined by the aria-relevant attribute.
	pub static aria_atomic: () = ();
	/// The [`aria-autocomplete`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-autocomplete) attribute indicates whether user input completion suggestions are provided.
	pub static aria_autocomplete: () = ();
	/// The [`aria-busy`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-busy) attribute indicates whether an element, and its subtree, are currently being updated.
	pub static aria_busy: () = ();
	/// The [`aria-checked`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-checked) attribute indicates the current "checked" state of checkboxes, radio buttons, and other widgets.
	pub static aria_checked: () = ();
	/// The [`aria-colcount`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-colcount) attribute defines the total number of columns in a table, grid, or treegrid.
	pub static aria_colcount: () = ();
	/// The [`aria-colindex`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-colindex) attribute defines an element's column index or position with respect to the total number of columns within a table, grid, or treegrid.
	pub static aria_colindex: () = ();
	/// The [`aria-colspan`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-colspan) attribute defines the number of columns spanned by a cell or gridcell within a table, grid, or treegrid.
	pub static aria_colspan: () = ();
	/// The [`aria-controls`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-controls) attribute identifies the element (or elements) whose contents or presence are controlled by the current element.
	pub static aria_controls: () = ();
	/// The [`aria-current`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-current) attribute indicates the element representing the current item within a container or set of related elements.
	pub static aria_current: () = ();
	/// The [`aria-describedby`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-describedby) attribute identifies the element (or elements) that describes the object.
	pub static aria_describedby: () = ();
	/// The [`aria-description`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-description) attribute provides a string value that describes or annotates the current element.
	pub static aria_description: () = ();
	/// The [`aria-details`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-details) attribute identifies the element that provides a detailed, extended description for the object.
	pub static aria_details: () = ();
	/// The [`aria-disabled`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-disabled) attribute indicates that the element is perceivable but disabled, so it is not editable or otherwise operable.
	pub static aria_disabled: () = ();
	/// The [`aria-dropeffect`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-dropeffect) attribute indicates what functions can be performed when a dragged object is released on the drop target.
	pub static aria_dropeffect: () = ();
	/// The [`aria-errormessage`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-errormessage) attribute identifies the element that provides an error message for the object.
	pub static aria_errormessage: () = ();
	/// The [`aria-expanded`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-expanded) attribute indicates whether an element, or another grouping element it controls, is currently expanded or collapsed.
	pub static aria_expanded: () = ();
	/// The [`aria-flowto`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-flowto) attribute identifies the next element (or elements) in an alternate reading order of content.
	pub static aria_flowto: () = ();
	/// The [`aria-grabbed`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-grabbed) attribute indicates an element's "grabbed" state in a drag-and-drop operation.
	pub static aria_grabbed: () = ();
	/// The [`aria-haspopup`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-haspopup) attribute indicates the availability and type of interactive popup element, such as menu or dialog, that can be triggered by an element.
	pub static aria_haspopup: () = ();
	/// The [`aria-hidden`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-hidden) attribute indicates whether the element is exposed to an accessibility API.
	pub static aria_hidden: () = ();
	/// The [`aria-invalid`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-invalid) attribute indicates the entered value does not conform to the format expected by the application.
	pub static aria_invalid: () = ();
	/// The [`aria-keyshortcuts`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-keyshortcuts) attribute indicates keyboard shortcuts that an author has implemented to activate or give focus to an element.
	pub static aria_keyshortcuts: () = ();
	/// The [`aria-label`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-label) attribute defines a string value that labels the current element.
	pub static aria_label: () = ();
	/// The [`aria-labelledby`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-labelledby) attribute identifies the element (or elements) that labels the current element.
	pub static aria_labelledby: () = ();
	/// The [`aria-live`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-live) attribute indicates that an element will be updated, and describes the types of updates the user agents, assistive technologies, and user can expect from the live region.
	pub static aria_live: () = ();
	/// The [`aria-modal`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-modal) attribute indicates whether an element is modal when displayed.
	pub static aria_modal: () = ();
	/// The [`aria-multiline`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-multiline) attribute indicates whether a text box accepts multiple lines of input or only a single line.
	pub static aria_multiline: () = ();
	/// The [`aria-multiselectable`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-multiselectable) attribute indicates that the user may select more than one item from the current selectable descendants.
	pub static aria_multiselectable: () = ();
	/// The [`aria-orientation`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-orientation) attribute indicates whether the element's orientation is horizontal, vertical, or unknown/ambiguous.
	pub static aria_orientation: () = ();
	/// The [`aria-owns`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-owns) attribute identifies an element (or elements) in order to define a relationship between the element with `aria-owns` and the target element.
	pub static aria_owns: () = ();
	/// The [`aria-placeholder`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-placeholder) attribute defines a short hint (a word or short phrase) intended to aid the user with data entry when the control has no value.
	pub static aria_placeholder: () = ();
	/// The [`aria-posinset`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-posinset) attribute defines an element's position within a set or treegrid.
	pub static aria_posinset: () = ();
	/// The [`aria-pressed`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-pressed) attribute indicates the current "pressed" state of toggle buttons.
	pub static aria_pressed: () = ();
	/// The [`aria-readonly`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-readonly) attribute indicates that the element is not editable, but is otherwise operable.
	pub static aria_readonly: () = ();
	/// The [`aria-relevant`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-relevant) attribute indicates what user agent changes to the accessibility tree should be monitored.
	pub static aria_relevant: () = ();
	/// The [`aria-required`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-required) attribute indicates that user input is required on the element before a form may be submitted.
	pub static aria_required: () = ();
	/// The [`aria-roledescription`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-roledescription) attribute defines a human-readable, author-localized description for the role of an element.
	pub static aria_roledescription: () = ();
	/// The [`aria-rowcount`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-rowcount) attribute defines the total number of rows in a table, grid, or treegrid.
	pub static aria_rowcount: () = ();
	/// The [`aria-rowindex`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-rowindex) attribute defines an element's row index or position with respect to the total number of rows within a table, grid, or treegrid.
	pub static aria_rowindex: () = ();
	/// The [`aria-rowspan`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-rowspan) attribute defines the number of rows spanned by a cell or gridcell within a table, grid, or treegrid.
	pub static aria_rowspan: () = ();
	/// The [`aria-selected`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-selected) attribute indicates the current "selected" state of various widgets.
	pub static aria_selected: () = ();
	/// The [`aria-setsize`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-setsize) attribute defines the number of items in the current set of listitems or treeitems.
	pub static aria_setsize: () = ();
	/// The [`aria-sort`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-sort) attribute indicates if items in a table or grid are sorted in ascending or descending order.
	pub static aria_sort: () = ();
	/// The [`aria-valuemax`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-valuemax) attribute defines the maximum allowed value for a range widget.
	pub static aria_valuemax: () = ();
	/// The [`aria-valuemin`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-valuemin) attribute defines the minimum allowed value for a range widget.
	pub static aria_valuemin: () = ();
	/// The [`aria-valuenow`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-valuenow) attribute defines the current value for a range widget.
	pub static aria_valuenow: () = ();
	/// The [`aria-valuetext`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-valuetext) attribute defines the human-readable text alternative of aria-valuenow for a range widget.
	pub static aria_valuetext: () = ();
	/// The `attributionsrc` attribute indicates that you want the browser to send an `Attribution-Reporting-Eligible` header along with a request.
	pub static attributionsrc: () = ();
	/// The [`autocapitalize`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/autocapitalize) attribute controls whether and how text input is automatically capitalized as it is entered/edited by the user.
	pub static autocapitalize: () = ();
	/// The `autocomplete` attribute indicates whether an input field can have its value automatically completed by the browser.
	pub static autocomplete: () = ();
	/// The [`autofocus`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/autofocus) attribute indicates that an element should be focused on page load.
	pub static autofocus: () = ();
	/// The `autoplay` attribute indicates that the media should start playing as soon as it is loaded.
	pub static autoplay: () = ();
	/// The `background` attribute sets the URL of the background image for the document.
	pub static background: () = ();
	/// The `bgcolor` attribute sets the background color of an element.
	pub static bgcolor: () = ();
	/// The `blocking` attribute indicates that the script will block the page loading until it is executed.
	pub static blocking: () = ();
	/// The `border` attribute sets the width of an element's border.
	pub static border: () = ();
	/// The `buffered` attribute contains the time ranges that the media has been buffered.
	pub static buffered: () = ();
	/// The `capture` attribute indicates that the user must capture media using a camera or microphone instead of selecting a file from the file picker.
	pub static capture: () = ();
	/// The `challenge` attribute specifies the challenge string that is paired with the keygen element.
	pub static challenge: () = ();
	/// The `charset` attribute specifies the character encoding of the HTML document.
	pub static charset: () = ();
	/// The `checked` attribute indicates whether an input element is checked or not.
	pub static checked: () = ();
	/// The `cite` attribute contains a URL that points to the source of the quotation or change.
	pub static cite: () = ();
	/// The [`class`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/class) attribute is a space-separated list of the classes of the element
	pub static class: () = ();
	/// The `closedby` attribute specifies the types of user actions that can be used to close the associated `<dialog>` element.
	pub static closedby: () = ();
	/// The `code` attribute specifies the URL of the applet's class file to be loaded and executed.
	pub static code: () = ();
	/// The `color` attribute specifies the color of an element's text.
	pub static color: () = ();
	/// The `cols` attribute specifies the visible width of a text area.
	pub static cols: () = ();
	/// The `colspan` attribute defines the number of columns a cell should span.
	pub static colspan: () = ();
	/// The `command` attribute defines the command to be invoked when user clicks the `<button>` element which has `commandfor` attribute specified.
	pub static command: () = ();
	/// The `commandfor` attribute defines the id of the element which button is controlling. It is generic version of `popovertarget`.
	pub static commandfor: () = ();
	/// The `content` attribute gives the value associated with the http-equiv or name attribute.
	pub static content: () = ();
	/// The [`contenteditable`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/contenteditable) attribute indicates whether the element's content is editable.
	pub static contenteditable: () = ();
	/// The `contextmenu` attribute specifies the ID of a `<menu>` element to open as a context menu.
	pub static contextmenu: () = ();
	/// The `controls` attribute indicates whether the browser should display playback controls for the media.
	pub static controls: () = ();
	/// The `controlslist` attribute allows the control of which controls to show on the media element whenever the browser shows its native controls.
	pub static controlslist: () = ();
	/// The `coords` attribute specifies the coordinates of an area in an image map.
	pub static coords: () = ();
	/// The `crossorigin` attribute indicates whether the resource should be fetched with a CORS request.
	pub static crossorigin: () = ();
	/// The `csp` attribute allows the embedding document to define the Content Security Policy that an embedded document must agree to enforce upon itself.
	pub static csp: () = ();
	/// The `data` attribute specifies the URL of the resource that is being embedded.
	pub static data: () = ();
	/// The `datetime` attribute specifies the date and time.
	pub static datetime: () = ();
	/// The `decoding` attribute indicates the preferred method for decoding images.
	pub static decoding: () = ();
	/// The `default` attribute indicates that the track should be enabled unless the user's preferences indicate that another track is more appropriate.
	pub static default: () = ();
	/// The `defer` attribute indicates that the script should be executed after the document has been parsed.
	pub static defer: () = ();
	/// The [`dir`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/dir) attribute specifies the text direction for the content in an element.
	pub static dir: () = ();
	/// The `dirname` attribute identifies the text directionality of an input element.
	pub static dirname: () = ();
	/// The `disabled` attribute indicates whether the element is disabled.
	pub static disabled: () = ();
	/// The `disablepictureinpicture` attribute indicates that the element is not allowed to be displayed in Picture-in-Picture mode.
	pub static disablepictureinpicture: () = ();
	/// The `disableremoteplayback` attribute indicates that the element is not allowed to be displayed using remote playback.
	pub static disableremoteplayback: () = ();
	/// The `download` attribute indicates that the linked resource is intended to be downloaded rather than displayed in the browser.
	pub static download: () = ();
	/// The [`draggable`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/draggable) attribute indicates whether the element is draggable.
	pub static draggable: () = ();
	/// The `elementtiming` attributes marks the element for observation by the `PerformanceElementTiming` API.
	pub static elementtiming: () = ();
	/// The `enctype` attribute specifies the MIME type of the form submission.
	pub static enctype: () = ();
	/// The [`enterkeyhint`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/enterkeyhint) attribute allows authors to specify what kind of action label or icon will be presented to users in a virtual keyboard's enter key.
	pub static enterkeyhint: () = ();
	/// The [`exportparts`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/exportparts) attribute enables the sharing of parts of an element's shadow DOM with a containing document.
	pub static exportparts: () = ();
	/// The `fetchpriority` attribute allows developers to specify the priority of a resource fetch request.
	pub static fetchpriority: () = ();
	/// The `form` attribute associates the element with a form element.
	pub static form: () = ();
	/// The `formaction` attribute specifies the URL that processes the form submission.
	pub static formaction: () = ();
	/// The `formenctype` attribute specifies how the form data should be encoded when submitted.
	pub static formenctype: () = ();
	/// The `formmethod` attribute specifies the HTTP method to use when submitting the form.
	pub static formmethod: () = ();
	/// The `formnovalidate` attribute indicates that the form should not be validated when submitted.
	pub static formnovalidate: () = ();
	/// The `formtarget` attribute specifies where to display the response after submitting the form.
	pub static formtarget: () = ();
	/// The `headers` attribute specifies the headers associated with the element.
	pub static headers: () = ();
	/// The `height` attribute specifies the height of an element.
	pub static height: () = ();
	/// The [`hidden`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/hidden) attribute indicates that the element is not yet, or is no longer, relevant.
	pub static hidden: () = ();
	/// The `high` attribute specifies the range that is considered to be a high value.
	pub static high: () = ();
	/// The `href` attribute specifies the URL of a linked resource.
	pub static href: () = ();
	/// The `hreflang` attribute specifies the language of the linked resource.
	pub static hreflang: () = ();
	/// The `http-equiv` attribute provides an HTTP header for the information/value of the content attribute.
	pub static http_equiv: () = ();
	/// The `icon` attribute specifies the URL of an image to be used as a graphical icon for the element.
	pub static icon: () = ();
	/// The [`id`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/id) attribute specifies a unique id for an element.
	pub static id: () = ();
	/// The `imagesizes` attribute specifies image sizes for different page layouts.
	pub static imagesizes: () = ();
	/// The `imagesrcset` attribute specifies the URLs of multiple images to be used in different situations.
	pub static imagesrcset: () = ();
	/// The `importance` attribute specifies the relative importance of the element.
	pub static importance: () = ();
	/// The [`inert`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/inert) attribute indicates that the element is non-interactive and won't be accessible to user interactions or assistive technologies.
	pub static inert: () = ();
	/// The [`inputmode`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/inputmode) attribute specifies the type of data that the user will enter.
	pub static inputmode: () = ();
	/// The `integrity` attribute contains a hash value that the browser can use to verify that the resource hasn't been altered.
	pub static integrity: () = ();
	/// The `intrinsicsize` attribute specifies the intrinsic size of an image or video.
	pub static intrinsicsize: () = ();
	/// The [`is`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/is) attribute allows you to specify the name of a custom element.
	pub static is: () = ();
	/// The `ismap` attribute indicates that the image is part of a server-side image map.
	pub static ismap: () = ();
	/// The [`itemid`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/itemid) attribute assigns a unique identifier to an item.
	pub static itemid: () = ();
	/// The [`itemprop`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/itemprop) attribute adds a property to an item.
	pub static itemprop: () = ();
	/// The [`itemref`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/itemref) attribute provides a list of element IDs that have additional properties for the item.
	pub static itemref: () = ();
	/// The [`itemscope`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/itemscope) attribute creates a new item and adds it to the page's items.
	pub static itemscope: () = ();
	/// The [`itemtype`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/itemtype) attribute specifies the type of an item.
	pub static itemtype: () = ();
	/// The `keytype` attribute specifies the type of key used by the `<keygen>` element.
	pub static keytype: () = ();
	/// The `kind` attribute specifies the kind of text track.
	pub static kind: () = ();
	/// The `label` attribute provides a user-readable title for an element.
	pub static label: () = ();
	/// The [`lang`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/lang) attribute specifies the language of the element's content.
	pub static lang: () = ();
	/// The `language` attribute specifies the scripting language used for the script.
	pub static language: () = ();
	/// The `list` attribute identifies a `<datalist>` element that contains pre-defined options for an `<input>` element.
	pub static list: () = ();
	/// The `loading` attribute indicates how the browser should load the image.
	pub static loading: () = ();
	/// The `low` attribute specifies the range that is considered to be a low value.
	pub static low: () = ();
	/// The `manifest` attribute specifies the URL of a document's cache manifest.
	pub static manifest: () = ();
	/// The `max` attribute specifies the maximum value for an input element.
	pub static max: () = ();
	/// The `maxlength` attribute specifies the maximum number of characters that an input element can accept.
	pub static maxlength: () = ();
	/// The `media` attribute specifies what media/device the linked resource is optimized for.
	pub static media: () = ();
	/// The `method` attribute specifies the HTTP method to use when submitting the form.
	pub static method: () = ();
	/// The `min` attribute specifies the minimum value for an input element.
	pub static min: () = ();
	/// The `minlength` attribute specifies the minimum number of characters that an input element can accept.
	pub static minlength: () = ();
	/// The `multiple` attribute indicates whether the user can enter more than one value.
	pub static multiple: () = ();
	/// The `muted` attribute indicates whether the audio will be initially silenced on page load.
	pub static muted: () = ();
	/// The `name` attribute specifies the name of the element.
	pub static name: () = ();
	/// The `nomodule` attribute indicates that the script should not be executed in browsers that support ES modules.
	pub static nomodule: () = ();
	/// The [`nonce`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/nonce) attribute provides a cryptographic nonce to ensure that a script or style is approved for execution.
	pub static nonce: () = ();
	/// The `novalidate` attribute indicates that the form should not be validated when submitted.
	pub static novalidate: () = ();
	/// The `open` attribute indicates whether the details element is open or closed.
	pub static open: () = ();
	/// The `optimum` attribute specifies the range that is considered to be an optimum value.
	pub static optimum: () = ();
	/// The [`part`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/part) attribute identifies the element as a shadow DOM part.
	pub static part: () = ();
	/// The `pattern` attribute specifies a regular expression that the input element's value is checked against.
	pub static pattern: () = ();
	/// The `ping` attribute contains a space-separated list of URLs to be notified if the user follows the hyperlink.
	pub static ping: () = ();
	/// The `placeholder` attribute provides a short hint that describes the expected value of the input element.
	pub static placeholder: () = ();
	/// The `playsinline` attribute indicates that the video should play inline in the element's playback area.
	pub static playsinline: () = ();
	/// The [`popover`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/popover) attribute indicates that an element is a popover and specifies the event that causes the popover to be shown.
	pub static popover: () = ();
	/// The `popovertarget` attribute specifies the ID of an element to toggle a popover.
	pub static popovertarget: () = ();
	/// The `popovertargetaction` attribute specifies the action that shows the popover.
	pub static popovertargetaction: () = ();
	/// The `poster` attribute specifies an image to be shown while the video is downloading or until the user hits the play button.
	pub static poster: () = ();
	/// The `preload` attribute specifies if and how the author thinks that the media file should be loaded when the page loads.
	pub static preload: () = ();
	/// The `as` attribute specifies the type of destination for the content of the link.
	pub static r#as: () = ();
	/// The `async` attribute indicates that the script should be executed asynchronously.
	pub static r#async: () = ();
	/// The `for` attribute specifies which form element a label is bound to.
	pub static r#for: () = ();
	/// The `loop` attribute indicates whether the media should start over again when it reaches the end.
	pub static r#loop: () = ();
	/// The `type` attribute specifies the type of the element.
	pub static r#type: () = ();
	/// The `radiogroup` attribute specifies the name of the group to which the element belongs.
	pub static radiogroup: () = ();
	/// The `readonly` attribute indicates that the user cannot modify the value of the input element.
	pub static readonly: () = ();
	/// The `referrerpolicy` attribute specifies which referrer information to include with requests.
	pub static referrerpolicy: () = ();
	/// The `rel` attribute specifies the relationship between the current document and the linked document.
	pub static rel: () = ();
	/// The `required` attribute indicates that the user must fill in the input element before submitting the form.
	pub static required: () = ();
	/// The `reversed` attribute indicates that the list should be displayed in a descending order.
	pub static reversed: () = ();
	/// The [`role`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/role) attribute defines the role of an element in the context of a web application.
	pub static role: () = ();
	/// The `rows` attribute specifies the number of visible text lines for a text area.
	pub static rows: () = ();
	/// The `rowspan` attribute defines the number of rows a cell should span.
	pub static rowspan: () = ();
	/// The `sandbox` attribute applies extra restrictions to the content in the `<iframe>`.
	pub static sandbox: () = ();
	/// The `scope` attribute specifies whether a header cell is a header for a column, row, or group of columns or rows.
	pub static scope: () = ();
	/// The `scoped` attribute indicates that the styles in a `<style>` element are scoped to the parent element.
	pub static scoped: () = ();
	/// The `selected` attribute indicates that the option is selected.
	pub static selected: () = ();
	/// The `shape` attribute specifies the shape of the area.
	pub static shape: () = ();
	/// The `size` attribute specifies the width of the input element.
	pub static size: () = ();
	/// The `sizes` attribute specifies the sizes of icons for visual media.
	pub static sizes: () = ();
	/// The [`slot`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/slot) attribute assigns a slot to an element.
	pub static slot: () = ();
	/// The `span` attribute defines the number of columns in a `<colgroup>` or the number of rows in a `<rowgroup>`.
	pub static span: () = ();
	/// The [`spellcheck`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/spellcheck) attribute indicates whether spell checking is allowed for the element.
	pub static spellcheck: () = ();
	/// The `src` attribute specifies the URL of the media resource.
	pub static src: () = ();
	/// The `srcdoc` attribute specifies the HTML content of the page to show in the `<iframe>`.
	pub static srcdoc: () = ();
	/// The `srclang` attribute specifies the language of the text track.
	pub static srclang: () = ();
	/// The `srcset` attribute specifies the URLs of multiple images to be used in different situations.
	pub static srcset: () = ();
	/// The `start` attribute specifies the start value of the list.
	pub static start: () = ();
	/// The `step` attribute specifies the legal number intervals for an input element.
	pub static step: () = ();
	/// The [`style`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/style) attribute specifies inline CSS styling declarations for an element.
	pub static style: () = ();
	/// The `summary` attribute provides a summary of the content of the table.
	pub static summary: () = ();
	/// The [`tabindex`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/tabindex) attribute specifies the tab order of an element.
	pub static tabindex: () = ();
	/// The `target` attribute specifies where to open the linked document.
	pub static target: () = ();
	/// The [`title`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/title) attribute provides additional information about an element.
	pub static title: () = ();
	/// The [`translate`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/translate) attribute specifies whether the content of an element should be translated or not.
	pub static translate: () = ();
	/// The `usemap` attribute specifies the image map to be used by an `<img>` element.
	pub static usemap: () = ();
	/// The `value` attribute specifies the value of the element.
	pub static value: () = ();
	/// The [`virtualkeyboardpolicy`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/virtualkeyboardpolicy) attribute controls the policy for virtual keyboards.
	pub static virtualkeyboardpolicy: () = ();
	/// The `width` attribute specifies the width of an element.
	pub static width: () = ();
	/// The `wrap` attribute specifies how the text in a text area is to be wrapped when submitted in a form.
	pub static wrap: () = ();
}

/// HTML events descriptions
pub mod html_events {
	/// The [`abort`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/abort_event) event is fired when the resource was not fully loaded, but not as the result of an error.
	pub fn abort() {}
	/// The [`animationcancel`](https://developer.mozilla.org/en-US/docs/Web/API/Element/animationcancel_event) event is fired when a CSS Animation unexpectedly aborts.
	pub fn animationcancel() {}
	/// The [`animationend`](https://developer.mozilla.org/en-US/docs/Web/API/Element/animationend_event) event is fired when a CSS Animation has completed.
	pub fn animationend() {}
	/// The [`animationiteration`](https://developer.mozilla.org/en-US/docs/Web/API/Element/animationiteration_event) event is fired when an iteration of a CSS Animation ends, and another one begins.
	pub fn animationiteration() {}
	/// The [`animationstart`](https://developer.mozilla.org/en-US/docs/Web/API/Element/animationstart_event) event is fired when a CSS Animation has started.
	pub fn animationstart() {}
	/// The [`auxclick`](https://developer.mozilla.org/en-US/docs/Web/API/Element/auxclick_event) event is fired at an `Element` when a non-primary pointing device button (any mouse button other than the primary—usually leftmost—button) has been pressed and released both within the same element.
	pub fn auxclick() {}
	/// The DOM [`beforeinput`](https://developer.mozilla.org/en-US/docs/Web/API/Element/beforeinput_event) event fires when the value of an `<input>` or `<textarea>` element is about to be modified.
	pub fn beforeinput() {}
	/// An element receives a [`beforematch`](https://developer.mozilla.org/en-US/docs/Web/API/Element/beforematch_event) event when it is in the hidden until found state and the browser is about to reveal its content because the user has found the content through the "find in page" feature or through fragment navigation.
	pub fn beforematch() {}
	/// The [`beforetoggle`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/beforetoggle_event) event of the `HTMLElement` interface fires on a popover or `<dialog>` element just before it is shown or hidden.
	pub fn beforetoggle() {}
	/// The [`blur`](https://developer.mozilla.org/en-US/docs/Web/API/Element/blur_event) event fires when an element has lost focus.
	pub fn blur() {}
	/// The [`cancel`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/cancel_event) event fires on an `<input>` element when the user cancels the file picker dialog via the Esc key or the cancel button and when the user re-selects the same files that were previously selected of `type="file"`.
	pub fn cancel() {}
	/// The [`canplay`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/canplay_event) event is fired when the user agent can play the media, but estimates that not enough data has been loaded to play the media up to its end without having to stop for further buffering of content.
	pub fn canplay() {}
	/// The [`canplaythrough`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/canplaythrough_event) event is fired when the user agent can play the media, and estimates that enough data has been loaded to play the media up to its end without having to stop for further buffering of content.
	pub fn canplaythrough() {}
	/// The [`change`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/change_event) event is fired for `<input>`, `<select>`, and `<textarea>` elements when the user modifies the element's value.
	pub fn change() {}
	/// An element receives a [`click`](https://developer.mozilla.org/en-US/docs/Web/API/Element/click_event) event when any of the following occurs:.
	pub fn click() {}
	/// The [`close`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/close_event) event is fired on an `HTMLDialogElement` object when the `<dialog>` it represents has been closed.
	pub fn close() {}
	/// The [`command`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/command_event) event of the `HTMLElement` interface fires on an element that is controlled via a `button` with valid `commandForElement` and `command` values, whenever the button is interacted with (e.g., it is clicked).
	pub fn command() {}
	/// The [`contentvisibilityautostatechange`](https://developer.mozilla.org/en-US/docs/Web/API/Element/contentvisibilityautostatechange_event) event fires on any element with `content-visibility: auto` set on it when it starts or stops being relevant to the user and skipping its contents.
	pub fn contentvisibilityautostatechange() {}
	/// The [`contextlost`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/contextlost_event) event of the Canvas API is fired if the user agent detects that the backing storage associated with a `CanvasRenderingContext2D` context is lost.
	pub fn contextlost() {}
	/// The [`contextmenu`](https://developer.mozilla.org/en-US/docs/Web/API/Element/contextmenu_event) event fires when the user attempts to open a context menu.
	pub fn contextmenu() {}
	/// The [`contextrestored`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/contextrestored_event) event of the Canvas API is fired if the user agent restores the backing storage for a `CanvasRenderingContext2D`.
	pub fn contextrestored() {}
	/// The [`copy`](https://developer.mozilla.org/en-US/docs/Web/API/Element/copy_event) event of the Clipboard API fires when the user initiates a copy action through the browser's user interface.
	pub fn copy() {}
	/// The [`cuechange`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLTrackElement/cuechange_event) event fires when a `TextTrack` has changed the currently displaying cues.
	pub fn cuechange() {}
	/// The [`cut`](https://developer.mozilla.org/en-US/docs/Web/API/Element/cut_event) event of the Clipboard API is fired when the user has initiated a "cut" action through the browser's user interface.
	pub fn cut() {}
	/// The [`dblclick`](https://developer.mozilla.org/en-US/docs/Web/API/Element/dblclick_event) event fires when a pointing device button (such as a mouse's primary button) is double-clicked; that is, when it's rapidly clicked twice on a single element within a very short span of time.
	pub fn dblclick() {}
	/// The [`drag`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/drag_event) event is fired every few hundred milliseconds as an element or text selection is being dragged by the user.
	pub fn drag() {}
	/// The [`dragend`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragend_event) event is fired when a drag operation ends (by releasing a mouse button or hitting the escape key).
	pub fn dragend() {}
	/// The [`dragenter`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragenter_event) event is fired when a dragged element or text selection enters a valid drop target.
	pub fn dragenter() {}
	/// The [`dragleave`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragleave_event) event is fired when a dragged element or text selection leaves a valid drop target.
	pub fn dragleave() {}
	/// The [`dragover`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragover_event) event is fired when an element or text selection is being dragged over a valid drop target (every few hundred milliseconds).
	pub fn dragover() {}
	/// The [`dragstart`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragstart_event) event is fired when the user starts dragging an element or text selection.
	pub fn dragstart() {}
	/// The [`drop`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/drop_event) event is fired when an element or text selection is dropped on a valid drop target.
	pub fn drop() {}
	/// The [`durationchange`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/durationchange_event) event is fired when the `duration` attribute has been updated.
	pub fn durationchange() {}
	/// The [`emptied`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/emptied_event) event is fired when the media has become empty; for example, this event is sent if the media has already been loaded (or partially loaded), and the `load()` method is called to reload it.
	pub fn emptied() {}
	/// The [`ended`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/ended_event) event is fired when playback or streaming has stopped because the end of the media was reached or because no further data is available.
	pub fn ended() {}
	/// The [`error`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/error_event) event is fired on an element when a resource failed to load, or can't be used.
	pub fn error() {}
	/// The [`focus`](https://developer.mozilla.org/en-US/docs/Web/API/Element/focus_event) event fires when an element has received focus.
	pub fn focus() {}
	/// The [`focusin`](https://developer.mozilla.org/en-US/docs/Web/API/Element/focusin_event) event fires when an element has received focus, after the `focus` event.
	pub fn focusin() {}
	/// The [`focusout`](https://developer.mozilla.org/en-US/docs/Web/API/Element/focusout_event) event fires when an element has lost focus, after the `blur` event.
	pub fn focusout() {}
	/// The [`formdata`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/formdata_event) event fires after the entry list representing the form's data is confned() {}
	pub fn formdata() {}
	/// The [`fullscreenchange`](https://developer.mozilla.org/en-US/docs/Web/API/Element/fullscreenchange_event) event is fired immediately after an `Element` switches into or out of fullscreen mode.
	pub fn fullscreenchange() {}
	/// The [`fullscreenerror`](https://developer.mozilla.org/en-US/docs/Web/API/Element/fullscreenerror_event) event is fired when the browser cannot switch to fullscreen mode.
	pub fn fullscreenerror() {}
	/// The [`gotpointercapture`](https://developer.mozilla.org/en-US/docs/Web/API/Element/gotpointercapture_event) event is fired when an element captures a pointer using `setPointerCapture()`.
	pub fn gotpointercapture() {}
	/// The [`input`](https://developer.mozilla.org/en-US/docs/Web/API/Element/input_event) event fires when the `value` of an `<input>`, `<select>`, or `<textarea>` element has been changed as a direct result of a user action (such as typing in a textbox or checking a checkbox).
	pub fn input() {}
	/// The [`invalid`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/invalid_event) event fires when a submittable element has been checked for validity and doesn't satisfy its constraints.
	pub fn invalid() {}
	/// The [`keydown`](https://developer.mozilla.org/en-US/docs/Web/API/Element/keydown_event) event is fired when a key is pressed.
	pub fn keydown() {}
	/// The [`keyup`](https://developer.mozilla.org/en-US/docs/Web/API/Element/keyup_event) event is fired when a key is released.
	pub fn keyup() {}
	/// The [`load`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/load_event) event fires for elements containing a resource when the resource has successfully loaded.
	pub fn load() {}
	/// The [`loadeddata`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/loadeddata_event) event is fired when the frame at the current playback position of the media has finished loading; often the first frame.
	pub fn loadeddata() {}
	/// The [`loadedmetadata`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/loadedmetadata_event) event is fired when the metadata has been loaded.
	pub fn loadedmetadata() {}
	/// The [`loadstart`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/loadstart_event) event is fired when the browser has started to load a resource.
	pub fn loadstart() {}
	/// The [`lostpointercapture`](https://developer.mozilla.org/en-US/docs/Web/API/Element/lostpointercapture_event) event is fired when a captured pointer is released.
	pub fn lostpointercapture() {}
	/// The [`mousedown`](https://developer.mozilla.org/en-US/docs/Web/API/Element/mousedown_event) event is fired at an `Element` when a pointing device button is pressed while the pointer is inside the element.
	pub fn mousedown() {}
	/// The [`mouseenter`](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseenter_event) event is fired at an `Element` when a pointing device (usually a mouse) is initially moved so that its hotspot is within the element at which the event was fired.
	pub fn mouseenter() {}
	/// The [`mouseleave`](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseleave_event) event is fired at an `Element` when the cursor of a pointing device (usually a mouse) is moved out of it.
	pub fn mouseleave() {}
	/// The [`mousemove`](https://developer.mozilla.org/en-US/docs/Web/API/Element/mousemove_event) event is fired at an element when a pointing device (usually a mouse) is moved while the cursor's hotspot is inside it.
	pub fn mousemove() {}
	/// The [`mouseout`](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseout_event) event is fired at an `Element` when a pointing device (usually a mouse) is used to move the cursor so that it is no longer contained within the element or one of its children.
	pub fn mouseout() {}
	/// The [`mouseover`](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseover_event) event is fired at an `Element` when a pointing device (such as a mouse or trackpad) is used to move the cursor onto the element or one of its child elements.
	pub fn mouseover() {}
	/// The [`mouseup`](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseup_event) event is fired at an `Element` when a button on a pointing device (such as a mouse or trackpad) is released while the pointer is located inside it.
	pub fn mouseup() {}
	/// The [`paste`](https://developer.mozilla.org/en-US/docs/Web/API/Element/paste_event) event of the Clipboard API is fired when the user has initiated a "paste" action through the browser's user interface.
	pub fn paste() {}
	/// The [`pause`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/pause_event) event is sent when a request to pause an activity is handled and the activity has entered its paused state, most commonly after the media has been paused through a call to the element's `pause()` method.
	pub fn pause() {}
	/// The [`play`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/play_event) event is fired when the `paused` property is changed from `true` to `false`, as a result of the `play` method, or the `autoplay` attribute.
	pub fn play() {}
	/// The [`playing`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/playing_event) event is fired after playback is first started, and whenever it is restarted.
	pub fn playing() {}
	/// The [`pointercancel`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointercancel_event) event is fired when the browser determines that there are unlikely to be any more pointer events, or if after the `pointerdown` event is fired, the pointer is then used to manipulate the viewport by panning, zooming, or scrolling.
	pub fn pointercancel() {}
	/// The [`pointerdown`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointerdown_event) event is fired when a pointer becomes active.
	pub fn pointerdown() {}
	/// The [`pointerenter`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointerenter_event) event fires when a pointing device is moved into the hit test boundaries of an element or one of its descendants, including as a result of a `pointerdown` event from a device that does not support hover (see `pointerdown`).
	pub fn pointerenter() {}
	/// The [`pointerleave`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointerleave_event) event is fired when a pointing device is moved out of the hit test boundaries of an element.
	pub fn pointerleave() {}
	/// The [`pointermove`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointermove_event) event is fired when a pointer changes coordinates, and the pointer has not been canceled by a browser touch-action.
	pub fn pointermove() {}
	/// The [`pointerout`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointerout_event) event is fired for several reasons including: pointing device is moved out of the hit test boundaries of an element; firing the `pointerup` event for a device that does not support hover (see `pointerup`); after firing the `pointercancel` event (see `pointercancel`); when a pen stylus leaves the hover range detectable by the digitizer.
	pub fn pointerout() {}
	/// The [`pointerover`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointerover_event) event is fired when a pointing device is moved into an element's hit test boundaries.
	pub fn pointerover() {}
	/// Secure context: This feature is available only in secure contexts (HTTPS), in some or all supporting browsers.
	pub fn pointerrawupdate() {}
	/// The [`pointerup`](https://developer.mozilla.org/en-US/docs/Web/API/Element/pointerup_event) event is fired when a pointer is no longer active.
	pub fn pointerup() {}
	/// The [`progress`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/progress_event) event is fired periodically as the browser loads a resource.
	pub fn progress() {}
	/// The [`ratechange`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/ratechange_event) event is fired when the playback rate has changed.
	pub fn ratechange() {}
	/// The [`reset`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/reset_event) event fires when a `<form>` is reset.
	pub fn reset() {}
	/// The [`resize`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLVideoElement/resize_event) event of the `HTMLVideoElement` interface fires when one or both of the `videoWidth` and `videoHeight` properties have just been updated.
	pub fn resize() {}
	/// The [`scroll`](https://developer.mozilla.org/en-US/docs/Web/API/Element/scroll_event) event fires when an element has been scrolled.
	pub fn scroll() {}
	/// The [`scrollend`](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollend_event) event fires when element scrolling has completed.
	pub fn scrollend() {}
	/// The [`securitypolicyviolation`](https://developer.mozilla.org/en-US/docs/Web/API/Element/securitypolicyviolation_event) event is fired when a Content Security Policy is violated.
	pub fn securitypolicyviolation() {}
	/// The [`seeked`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/seeked_event) event is fired when a seek operation completed, the current playback position has changed, and the Boolean `seeking` attribute is changed to `false`.
	pub fn seeked() {}
	/// The [`seeking`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/seeking_event) event is fired when a seek operation starts, meaning the Boolean `seeking` attribute has changed to `true` and the media is seeking a new position.
	pub fn seeking() {}
	/// The [`select`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/select_event) event fires when some text has been selected.
	pub fn select() {}
	/// The [`selectionchange`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/selectionchange_event) event of the Selection API is fired when the text selection within an `<input>` element is changed.
	pub fn selectionchange() {}
	/// The [`selectstart`](https://developer.mozilla.org/en-US/docs/Web/API/Node/selectstart_event) event of the Selection API is fired when a user starts a new selection.
	pub fn selectstart() {}
	/// The [`slotchange`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLSlotElement/slotchange_event) event is fired on an `HTMLSlotElement` instance (`<slot>` element) when the node(s) contained in that slot change.
	pub fn slotchange() {}
	/// The [`stalled`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/stalled_event) event is fired when the user agent is trying to fetch media data, but data is unexpectedly not forthcoming.
	pub fn stalled() {}
	/// The [`submit`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/submit_event) event fires when a `<form>` is submitted.
	pub fn submit() {}
	/// The [`suspend`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/suspend_event) event is fired when the user agent is intentionally not fetching media data, in which case `HTMLMediaElement.networkState` is set to `HTMLMediaElement.NETWORK_IDLE`.
	pub fn suspend() {}
	/// The [`timeupdate`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/timeupdate_event) event is fired when the time indicated by the `currentTime` attribute has been updated.
	pub fn timeupdate() {}
	/// The [`toggle`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/toggle_event) event of the `HTMLElement` interface fires on a popover element, `<dialog>` element, or `<details>` element just after it is shown or hidden.
	pub fn toggle() {}
	/// The [`touchcancel`](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchcancel_event) event is fired when one or more touch points have been disrupted in an implementation-specific manner.
	pub fn touchcancel() {}
	/// The [`touchend`](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchend_event) event fires when one or more touch points are removed from the touch surface.
	pub fn touchend() {}
	/// The [`touchmove`](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchmove_event) event is fired when one or more touch points are moved along the touch surface.
	pub fn touchmove() {}
	/// The [`touchstart`](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchstart_event) event is fired when one or more touch points are placed on the touch surface.
	pub fn touchstart() {}
	/// The [`transitioncancel`](https://developer.mozilla.org/en-US/docs/Web/API/Element/transitioncancel_event) event is fired when a CSS transition is canceled.
	pub fn transitioncancel() {}
	/// The [`transitionend`](https://developer.mozilla.org/en-US/docs/Web/API/Element/transitionend_event) event is fired when a CSS transition has completed.
	pub fn transitionend() {}
	/// The [`transitionrun`](https://developer.mozilla.org/en-US/docs/Web/API/Element/transitionrun_event) event is fired when a CSS transition is first created, i.e., before any `transition-delay` has begun.
	pub fn transitionrun() {}
	/// The [`transitionstart`](https://developer.mozilla.org/en-US/docs/Web/API/Element/transitionstart_event) event is fired when a CSS transition has actually started, i.e., after any `transition-delay` has ended.
	pub fn transitionstart() {}
	/// The [`volumechange`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/volumechange_event) event is fired when either the `volume` attribute or the `muted` attribute has changed.
	pub fn volumechange() {}
	/// The [`waiting`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/waiting_event) event is fired when playback has stopped because of a temporary lack of data.
	pub fn waiting() {}
	/// The [`wheel`](https://developer.mozilla.org/en-US/docs/Web/API/Element/wheel_event) event fires when the user rotates a wheel button on a pointing device (typically a mouse).
	pub fn wheel() {}
}

/// CSS properties descriptions
#[cfg(feature = "css-types")]
pub use crate::css_props;
