// All process_html tests should have assert_eq! on the final output
// so the expected HTML is clearly visible for review.
use super::*;

fn process_html_test(frag: &str) -> String {
    process_html(frag, "https://example.com")
}

fn has_class(html: &str, class: &str) -> bool {
    html.contains(&format!("class=\"{}\"", class))
        || html.contains(&format!("class=\"{} ", class))
        || html.contains(&format!(" {}\"", class))
        || html.contains(&format!(" {} ", class))
}

#[test]
fn test_simple_sentences() {
    let input = "<p>Call me Ishmael. Some years ago I thought I would sail.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(
        output.contains("Call me Ishmael."),
        "should contain first sentence: {}",
        output
    );
    assert!(
        output.contains("Some years ago I thought I would sail."),
        "should contain second sentence: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">Call me Ishmael.</span><span class=\"tts_para_1\"> Some years ago I thought I would sail.</span></p> </div>");
}

#[test]
fn test_inline_elements_preserved() {
    let input = "<p>This is a sentence with <em>emphasis. And it continues</em> into the next sentence!</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(
        output.contains("<em>"),
        "should preserve em element: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">This is a sentence with <em>emphasis.</em></span><span class=\"tts_para_1\"><em> And it continues</em> into the next sentence!</span></p> </div>");
}

#[test]
fn test_multiple_inline_elements() {
    let input = "<p>First sentence <strong>with bold. Second sentence</strong> without bold.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(
        output.contains("<strong>"),
        "should preserve strong element: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">First sentence <strong>with bold.</strong></span><span class=\"tts_para_1\"><strong> Second sentence</strong> without bold.</span></p> </div>");
}

#[test]
fn test_abbreviations_not_split() {
    let input = "<p>Dr. Smith went to the U.S.A. and met Mr. Jones. Then he left.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(
        output.contains("Dr. Smith went to the U.S.A. and met Mr. Jones."),
        "should not split at abbreviations: {}",
        output
    );
    assert!(
        output.contains("Then he left."),
        "should have second sentence: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">Dr. Smith went to the U.S.A. and met Mr. Jones.</span><span class=\"tts_para_1\"> Then he left.</span></p> </div>");
}

#[test]
fn test_skip_tags_removed() {
    let input = "<p>Visible text.</p><script>alert('hidden');</script><p>More visible.</p>";
    let output = process_html_test(input);
    assert!(
        !output.contains("alert"),
        "script should be removed: {}",
        output
    );
    assert!(
        output.contains("Visible text."),
        "should contain first sentence: {}",
        output
    );
    assert!(
        output.contains("More visible."),
        "should contain second sentence: {}",
        output
    );
    assert_eq!(output, "<div> <p class=\"tts_para_0\">Visible text.</p><p class=\"tts_para_1\">More visible.</p> </div>");
}

#[test]
fn test_code_block_with_newlines() {
    let input =
        "<pre><code>function hello() {\n  console.log(\"world\");\n  return true;\n}</code></pre>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_code_block\"><span class=\"tts_para_0\">function hello() {\n</span><span class=\"tts_para_1\">  console.log(\"world\");\n</span><span class=\"tts_para_2\">  return true;\n</span><span class=\"tts_para_3\">}</span></pre> </div>");
}

#[test]
fn test_code_inline_no_element_children_flattened() {
    let input = "<p>Here is code: <code>let x = 1;</code></p>";
    let output = process_html_test(input);
    assert!(
        !has_class(&output, "tts_code_block"),
        "should NOT have tts_code_block for inline code without element children: {}",
        output
    );
    assert!(
        output.contains("let x = 1;"),
        "should preserve code content: {}",
        output
    );
    assert_eq!(
        output,
        "<div> <p class=\"tts_para_0\">Here is code: <code>let x = 1;</code></p> </div>"
    );
}

#[test]
fn test_semicolon_boundary() {
    let input = "<p>First clause; second clause; third clause.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_2"),
        "should have tts_para_2: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">First clause;</span><span class=\"tts_para_1\"> second clause;</span><span class=\"tts_para_2\"> third clause.</span></p> </div>");
}

#[test]
fn test_no_punctuation_single_sentence() {
    let input = "<div>Just some text without any punctuation marks</div>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        output.contains("Just some text without any punctuation marks"),
        "should contain the text: {}",
        output
    );
    assert_eq!(
        output,
        "<div> <div class=\"tts_para_0\">Just some text without any punctuation marks</div> </div>"
    );
}

#[test]
fn test_long_sentence_split() {
    let text = "word ".repeat(200);
    let input = format!("<p>{}</p>", text);
    let output = process_html_test(&input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    let expected = "<div> <p><span class=\"tts_para_0\">word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word</span><span class=\"tts_para_1\">word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word </span></p> </div>";
    assert_eq!(output, expected);
}

#[test]
fn test_empty_paragraph() {
    let input = "<p></p>";
    let output = process_html_test(input);
    assert_eq!(output, "<div> <p></p> </div>");
}

#[test]
fn test_anchor_url_resolution() {
    let input = "<p>Visit <a href=\"/about\">our page</a>.</p>";
    let output = process_html_test(input);
    assert!(
        output.contains("https://example.com/about"),
        "should resolve relative URL: {}",
        output
    );
    assert!(
        has_class(&output, "tts_anchor"),
        "should have tts_anchor class: {}",
        output
    );
    assert_eq!(output, "<div> <p class=\"tts_para_0\">Visit <a href=\"https://example.com/about\" class=\"tts_anchor\">our page</a>.</p> </div>");
}

#[test]
fn test_img_url_resolution() {
    let input = "<p><img src=\"/image.png\" alt=\"test\"></p>";
    let output = process_html_test(input);
    assert!(
        output.contains("https://example.com/image.png"),
        "should resolve img src: {}",
        output
    );
    assert_eq!(
        output,
        "<div> <p><img src=\"https://example.com/image.png\" alt=\"test\"></p> </div>"
    );
}

#[test]
fn test_blockquote_sentences() {
    let input = "<blockquote>First quote. Second quote.</blockquote>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert_eq!(output, "<div> <blockquote><span class=\"tts_para_0\">First quote.</span><span class=\"tts_para_1\"> Second quote.</span></blockquote> </div>");
}

#[test]
fn test_list_items() {
    let input = "<ul><li>First item. Second sentence.</li><li>Another item.</li></ul>";
    let output = process_html_test(input);
    assert!(
        output.contains("tts_para_"),
        "should have tts_para classes: {}",
        output
    );
    assert_eq!(output, "<div> <ul><li><span class=\"tts_para_0\">First item.</span><span class=\"tts_para_1\"> Second sentence.</span></li><li class=\"tts_para_2\">Another item.</li></ul> </div>");
}

#[test]
fn test_heading_sentences() {
    let input = "<h1>Title here. Subtitle here.</h1>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert_eq!(output, "<div> <h1><span class=\"tts_para_0\">Title here.</span><span class=\"tts_para_1\"> Subtitle here.</span></h1> </div>");
}

#[test]
fn test_nested_inline_elements() {
    let input = "<p>Text <em>with <strong>bold. More</strong> text</em> here.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(output.contains("<em>"), "should preserve em: {}", output);
    assert!(
        output.contains("<strong>"),
        "should preserve strong: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">Text <em>with <strong>bold.</strong></em></span><span class=\"tts_para_1\"><em><strong> More</strong> text</em> here.</span></p> </div>");
}

#[test]
fn test_code_block_no_newlines_no_element_children() {
    let input = "<pre><code>let a = 1; let b = 2; let c = 3;</code></pre>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "pre should get tts_para_0: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_para_0 tts_code_block\"><code>let a = 1; let b = 2; let c = 3;</code></pre> </div>");
}

#[test]
fn test_code_block_split_at_braces() {
    let input = "<pre><code>if (true) {\n  doSomething();\n}</code></pre>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_code_block\"><span class=\"tts_para_0\">if (true) {\n</span><span class=\"tts_para_1\">  doSomething();\n</span><span class=\"tts_para_2\">}</span></pre> </div>");
}

#[test]
fn test_code_block_split_at_line_comment() {
    let input = "<pre><code>let x = 1; // this is a comment\nlet y = 2;</code></pre>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_code_block\"><span class=\"tts_para_0\">let x = 1;</span><span class=\"tts_para_1\"> // this is a comment\n</span><span class=\"tts_para_2\">let y = 2;</span></pre> </div>");
}

#[test]
fn test_multiple_block_elements() {
    let input = "<h1>Title.</h1><p>First paragraph. Second sentence.</p><p>Another paragraph.</p>";
    let output = process_html_test(input);
    assert!(
        output.contains("tts_para_"),
        "should have tts_para classes: {}",
        output
    );
    assert!(
        output.contains("Title."),
        "should contain title: {}",
        output
    );
    assert!(
        output.contains("First paragraph."),
        "should contain first paragraph: {}",
        output
    );
    assert!(
        output.contains("Another paragraph."),
        "should contain another paragraph: {}",
        output
    );
    assert_eq!(output, "<div> <h1 class=\"tts_para_0\">Title.</h1><p><span class=\"tts_para_1\">First paragraph.</span><span class=\"tts_para_2\"> Second sentence.</span></p><p class=\"tts_para_3\">Another paragraph.</p> </div>");
}

#[test]
fn test_exclamation_mark_boundary() {
    let input = "<p>Hello world! How are you? I am fine.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_2"),
        "should have tts_para_2: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">Hello world!</span><span class=\"tts_para_1\"> How are you?</span><span class=\"tts_para_2\"> I am fine.</span></p> </div>");
}

#[test]
fn test_question_mark_boundary() {
    let input = "<p>What is this? It is a test.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">What is this?</span><span class=\"tts_para_1\"> It is a test.</span></p> </div>");
}

#[test]
fn test_whitespace_only_paragraph() {
    let input = "<p>   </p>";
    let output = process_html_test(input);
    assert_eq!(output, "<div> <p>   </p> </div>");
}

#[test]
fn test_mixed_content_with_divs() {
    let input = "<div><p>Paragraph one.</p><p>Paragraph two.</p></div>";
    let output = process_html_test(input);
    assert!(
        output.contains("Paragraph one."),
        "should contain paragraph one: {}",
        output
    );
    assert!(
        output.contains("Paragraph two."),
        "should contain paragraph two: {}",
        output
    );
    assert_eq!(output, "<div> <div><p class=\"tts_para_0\">Paragraph one.</p><p class=\"tts_para_1\">Paragraph two.</p></div> </div>");
}

#[test]
fn test_sentence_boundary_with_comma() {
    let input = "<p>This is a long sentence, and it continues here, and ends here.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert_eq!(output, "<div> <p class=\"tts_para_0\">This is a long sentence, and it continues here, and ends here.</p> </div>");
}

#[test]
fn test_div_without_punctuation() {
    let input = "<div>Some text without punctuation</div>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        output.contains("Some text without punctuation"),
        "should contain text: {}",
        output
    );
    assert_eq!(
        output,
        "<div> <div class=\"tts_para_0\">Some text without punctuation</div> </div>"
    );
}

#[test]
fn test_preserves_html_structure() {
    let input = "<article><h1>Title.</h1><p>Content here.</p></article>";
    let output = process_html_test(input);
    assert!(
        output.contains("<article"),
        "should preserve article: {}",
        output
    );
    assert!(output.contains("<h1"), "should preserve h1: {}", output);
    assert!(output.contains("<p"), "should preserve p: {}", output);
    assert_eq!(output, "<div> <article><h1 class=\"tts_para_0\">Title.</h1><p class=\"tts_para_1\">Content here.</p></article> </div>");
}

#[test]
fn test_link_inside_sentence() {
    let input = "<p>Click <a href=\"/page\">here</a> for more info. Thanks.</p>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(output.contains("<a"), "should preserve anchor: {}", output);
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">Click <a href=\"https://example.com/page\" class=\"tts_anchor\">here</a> for more info.</span><span class=\"tts_para_1\"> Thanks.</span></p> </div>");
}

#[test]
fn test_fragment_only_url() {
    let input = "<p><a href=\"#section1\">Go to section</a></p>";
    let output = process_html_test(input);
    assert!(
        output.contains("#section1"),
        "should keep fragment-only URL: {}",
        output
    );
    assert!(
        !has_class(&output, "tts_anchor"),
        "fragment-only should not have tts_anchor: {}",
        output
    );
    assert_eq!(
        output,
        "<div> <p class=\"tts_para_0\"><a href=\"#section1\">Go to section</a></p> </div>"
    );
}

#[test]
fn test_bare_text_mixed_with_block_children() {
    let input = "<div>First sentence. Second sentence.<p>Block paragraph.</p></div>";
    let output = process_html_test(input);
    assert!(
        output.contains("First sentence."),
        "should contain bare text first sentence: {}",
        output
    );
    assert!(
        output.contains("Second sentence."),
        "should contain bare text second sentence: {}",
        output
    );
    assert!(
        output.contains("Block paragraph."),
        "should contain block paragraph: {}",
        output
    );
    assert_eq!(output, "<div> <div><span class=\"tts_para_0\">First sentence.</span><span class=\"tts_para_1\"> Second sentence.</span><p class=\"tts_para_2\">Block paragraph.</p></div> </div>");
}

#[test]
fn test_bare_text_before_blocks() {
    let input = "<div>Loose text.<h1>Title.</h1><p>Paragraph.</p></div>";
    let output = process_html_test(input);
    assert!(
        output.contains("Loose text."),
        "should contain loose text before blocks: {}",
        output
    );
    assert!(
        output.contains("Title."),
        "should contain title: {}",
        output
    );
    assert!(
        output.contains("Paragraph."),
        "should contain paragraph: {}",
        output
    );
    assert_eq!(output, "<div> <div><span class=\"tts_para_0\">Loose text.</span><h1 class=\"tts_para_1\">Title.</h1><p class=\"tts_para_2\">Paragraph.</p></div> </div>");
}

#[test]
fn test_single_bare_sentence_gets_tts_para() {
    let input = "<div>First sentence here. Second sentence here.<p>Block paragraph.</p></div>";
    let output = process_html_test(input);
    assert!(
        output.contains("First sentence here."),
        "should contain first bare sentence: {}",
        output
    );
    assert!(
        output.contains("Second sentence here."),
        "should contain second bare sentence: {}",
        output
    );
    assert_eq!(output, "<div> <div><span class=\"tts_para_0\">First sentence here.</span><span class=\"tts_para_1\"> Second sentence here.</span><p class=\"tts_para_2\">Block paragraph.</p></div> </div>");
}

#[test]
fn test_code_block_preserves_tabs_and_empty_lines() {
    let input = "<pre><code>fn main() {\n\tlet x = 1;\n\n\tlet y = 2;\n}</code></pre>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert!(
        output.contains("\tlet x = 1;"),
        "should preserve tab indentation: {}",
        output
    );
    assert!(
        output.contains("\tlet y = 2;"),
        "should preserve tab on second let: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_code_block\"><span class=\"tts_para_0\">fn main() {\n</span><span class=\"tts_para_1\">\tlet x = 1;\n</span><span class=\"tts_para_2\">\n</span><span class=\"tts_para_3\">\tlet y = 2;\n</span><span class=\"tts_para_4\">}</span></pre> </div>");
}

#[test]
fn test_li_with_inline_and_block_children() {
    let input = "<li><strong>Bold text.</strong><ul><li>Nested.</li></ul></li>";
    let output = process_html_test(input);
    assert_eq!(output, "<div> <li><strong class=\"tts_para_0\">Bold text.</strong><ul><li class=\"tts_para_1\">Nested.</li></ul></li> </div>");
}

#[test]
fn test_li_with_multisentence_inline_and_block_children() {
    let input = "<li><strong>First sentence. Second.</strong><ul><li>Nested.</li></ul></li>";
    let output = process_html_test(input);
    assert_eq!(output, "<div> <li><strong><span class=\"tts_para_0\">First sentence.</span><span class=\"tts_para_1\"> Second.</span></strong><ul><li class=\"tts_para_2\">Nested.</li></ul></li> </div>");
}

#[test]
fn test_pre_with_language_class() {
    let input = "<pre class=\"language-scala\" tabindex=\"0\"><code class=\"language-scala\">object Logger\n</code></pre>";
    let output = process_html_test(input);
    assert!(
        output.contains("tts_para_"),
        "should have tts_para: {}",
        output
    );
    assert!(
        output.contains("tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert!(
        output.contains("language-scala"),
        "should preserve language-scala class: {}",
        output
    );
    assert!(
        output.contains("tabindex"),
        "should preserve tabindex: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_para_0 language-scala tts_code_block\" tabindex=\"0\"><code class=\"language-scala\">object Logger\n</code></pre> </div>");
}

#[test]
fn test_table_gets_tts_para() {
    let input = "<table><thead><tr><th>Feature</th><th>session</th></tr></thead><tbody><tr><td>Works</td><td>works</td></tr></tbody></table>";
    let output = process_html_test(input);
    assert_eq!(output, "<div> <table><thead><tr><th class=\"tts_para_0\">Feature</th><th class=\"tts_para_1\">session</th></tr></thead><tbody><tr><td class=\"tts_para_2\">Works</td><td class=\"tts_para_3\">works</td></tr></tbody></table> </div>");
}

#[test]
fn test_table_multi_sentence_cell() {
    let input = "<table><tr><td>First sentence. Second sentence.</td></tr></table>";
    let output = process_html_test(input);
    assert_eq!(output, "<div> <table><tbody><tr><td><span class=\"tts_para_0\">First sentence.</span><span class=\"tts_para_1\"> Second sentence.</span></td></tr></tbody></table> </div>");
}

#[test]
fn test_nonstandard_wrapper_with_block_child() {
    let input = "<topcomment><article><p>First paragraph. </p><p>Second paragraph.</p></article></topcomment>";
    let output = process_html_test(input);
    assert!(
        output.contains("First paragraph."),
        "should contain first paragraph: {}",
        output
    );
    assert!(
        output.contains("Second paragraph."),
        "should contain second paragraph: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
}

#[test]
fn test_code_with_strong_inline_is_inline() {
    let input = "<p>This is an <code><strong>expert</strong></code> archer.</p>";
    let output = process_html_test(input);
    assert!(
        !has_class(&output, "tts_code_block"),
        "should NOT have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert_eq!(output, "<div> <p class=\"tts_para_0\">This is an <code><strong>expert</strong></code> archer.</p> </div>");
}

#[test]
fn test_code_with_sup_inline_is_inline() {
    let input = "<p>Text <code><sup>2</sup></code> more text.</p>";
    let output = process_html_test(input);
    assert!(
        !has_class(&output, "tts_code_block"),
        "should NOT have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert_eq!(
        output,
        "<div> <p class=\"tts_para_0\">Text <code><sup>2</sup></code> more text.</p> </div>"
    );
}

#[test]
fn test_code_with_nested_inline_is_inline() {
    let input = "<p>Text <code><strong><em>expert</em></strong></code> end.</p>";
    let output = process_html_test(input);
    assert!(
        !has_class(&output, "tts_code_block"),
        "should NOT have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert_eq!(output, "<div> <p class=\"tts_para_0\">Text <code><strong><em>expert</em></strong></code> end.</p> </div>");
}

#[test]
fn test_code_with_multisentence_inline_formatting() {
    let input = "<p>First sentence. <code><em>second.</em></code> Third sentence.</p>";
    let output = process_html_test(input);
    assert!(
        !has_class(&output, "tts_code_block"),
        "should NOT have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_2"),
        "should have tts_para_2: {}",
        output
    );
    assert_eq!(output, "<div> <p><span class=\"tts_para_0\">First sentence.</span><span class=\"tts_para_1\"> <code><em>second.</em></code></span><span class=\"tts_para_2\"> Third sentence.</span></p> </div>");
}

#[test]
fn test_pre_with_code_child_still_code_block() {
    let input = "<pre><code><strong>still code</strong></code></pre>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_para_0 tts_code_block\"><code><strong>still code</strong></code></pre> </div>");
}

#[test]
fn test_code_with_newline_and_inline_children_is_code_block() {
    let input = "<pre><code><strong>line1\nline2</strong></code></pre>";
    let output = process_html_test(input);
    assert!(
        has_class(&output, "tts_code_block"),
        "should have tts_code_block: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_0"),
        "should have tts_para_0: {}",
        output
    );
    assert!(
        has_class(&output, "tts_para_1"),
        "should have tts_para_1: {}",
        output
    );
    assert_eq!(output, "<div> <pre class=\"tts_code_block\"><span class=\"tts_para_0\">line1\n</span><span class=\"tts_para_1\">line2</span></pre> </div>");
}
