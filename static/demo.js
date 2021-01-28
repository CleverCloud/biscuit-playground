import * as wasm from "./wasm.js"

wasm.default()

window.block_count = 0;

function add_block() {
  let id = window.block_count;
  let block_list = document.getElementById("block-list");

  var element = document.createElement("li");
  element.class = "sub-container";

  var block_name;
  var should_display;
  if(id == 0) {
    block_name = "Authority block";
    should_display = "display: none";
  } else {
    block_name = "Block " + id;
    should_display = "";
  }

  element.innerHTML = `
      <div class="block" id="block-${id}">
          <div>
              <span>
                  <button onclick="delete_block(${id})"
                      style="${should_display}"
                  >-</button>
                  <h4 style="display:inline">${ block_name }</h4>
              </span>
              <br />
          </div>

          <textarea
              class="code"
              id=${ "block-code-" + id }
              oninput="contentUpdate()"
          ></textarea>
      </div>`;

  block_list.appendChild(element);
  window.block_count += 1;
}

add_block();
window.add_block = add_block;

function delete_block(id) {
  let block = document.getElementById("block-"+id);
  let li = block.parentNode;
  li.parentNode.removeChild(li);
  window.block_count -= 1;
}
window.delete_block = delete_block;

function contentUpdate() {
  /*var elements = document.getElementsByClassName('code');
  for (var i=0, len=elements.length|0; i<len; i=i+1|0) {
    console.log(elements[i].value);
  }*/
  console.log("will call testBiscuit");
  wasm.testBiscuit()

}

window.contentUpdate = contentUpdate;

function load() {
  add_block();
  var authority = document.getElementById("block-code-0");
  authority.value = `// this is a fact, the basic data used in Datalog
// you can see it as one line in the "right" table
// facts with #authority can only be in the first block
right(#authority, "/folder1/file1", #read)
right(#authority, "/folder1/file1", #write)
right(#authority, "/folder2/file1", #read)

// rules generate more facts from existing ones. You can
// read it as "generate can_read with the content $file
// if there exists a 'right' fact that matches"
can_read($file) <- right(#authority, $file, #read)

// this is a check. It will succeed if it finds matching
// facts. Otherwise, the token validation will fail
check if operation(#ambient, #read)
`;

  var block1 = document.getElementById("block-code-1");
  block1.value = `// to restrict rights, we add blocks with more checks
// checks and rules can also test expressions along with
// the presence of facts. To match, the expression must
// evaluate to true
// #ambient indicates facts provided by the verifier to
// evaluate the request, like which resource is accessed,
// or what is the current date
check if
  resource(#ambient, $file),
  $file.starts_with("/folder1/")
`;

  var verifier = document.getElementById("verifier-code");
  verifier.value = `// here we got a read request on /folder1/file1
resource(#ambient, "/folder1/file1")
operation(#ambient, #read)

// if this matches, the verification will succeed
allow if
  resource(#ambient, $file),
  operation(#ambient, $op),
  right(#authority, $file, $op)

// this catch-all policy will refuse the request
deny if true
`;

}

load();

window.editors = {};
window.marks = [];

function setup_codemirror() {
  var current_editors = Object.keys(window.editors);
  var new_editors = [];

  var elements = document.getElementsByClassName('code');
  for (var i=0, len=elements.length|0; i<len; i=i+1|0) {
    var editor = elements[i];
    new_editors.push(editor.id);

    if(!current_editors.includes(editor.id)) {
      let cm = new CodeMirror.fromTextArea(document.getElementById(editor.id), {
        mode: 'biscuit',
        autoCloseTags: true,
        lineNumbers: true
      });

      let id = editor.id;

      function updateTextArea() {
        // somehow save() does not work
        //cm.save();
        var text = cm.getValue();

        let editor = document.getElementById(id);
        editor.innerText = text;
        editor.value = text;

        const event = new Event('input');
        editor.dispatchEvent(event);
      }
      cm.on('change', updateTextArea);

      //cm.addLineClass(3, "text", "caveat_success");
      //var mark = cm.markText({line: 2, ch:2}, {line:3, ch:5}, { css: "background: #c1f1c1;" });
      window.editors[editor.id] = cm;
      //window.marks.push(mark);
    }
    //result += "\n  " + allOrangeJuiceByClass[i].textContent;
  }

  for(var i=0; i < current_editors.length; i=i+1) {
    if(!new_editors.includes(current_editors[i])) {
      delete window.editors[current_editors[i]];
    }
  }

  //console.log(window.editors);
}

setup_codemirror();
setTimeout(function() {
        const event = new Event('input');
        document.getElementById('verifier-code').dispatchEvent(event);
}, 500);
setInterval(setup_codemirror, 2000);
