# Vale

[Vale](https://vale.sh/) is a linter for prose.

* Vale understands Markdown and, for example, can spell check text but skip code blocks.
* Vale can also find Markdown comments in Rust code and lint them.
* Vale configuration can have custom lists of allowed and disallowed words for a project.

## Installation

Follow [the Vale installation instructions](https://vale.sh/docs/install).

Vale lists integrations in their documentation, including VS Code, JetBrains, and Neovim.
These plugins highlight errors in your editor in real time.
In general, Vale can show errors all over your project, but with the editor plugins you get more focused errors in the code that you edit.

Ensure that you can see errors in both Markdown files and Rust comments by forcing errors.

## Usage

Use the `.vale/check` script to run Vale on all Markdown and Rust files in the project.
Refer to the script documentation for variants to summarize and locate errors in bulk.

Vale is not infallible, errors can be wrong and addressing them might require different procedures.

Review our writing guide on [code blocks and spans](https://github.com/veecle/company-guidelines/blob/main/writing.md#code-blocks-and-spans) first, to learn how to use code blocks and spans.
Vale does not validate code blocks and spans, so using code blocks and spans correctly is key to Vale not reporting noisy false positives.
On the other hand, overusing code blocks and spans can be incorrect, preventing Vale from finding errors, reducing readability.

When you find an error you believe is incorrect, decide whether:

* The term must be written inside a code block or span.
  Refer to our writing guide on when this is the option.

* The term is correct outside code blocks and spans.
  Review the proper way to write the term (capitalization or others), typically by locating an authoritative source.
  Then, add the term to `.vale/styles/config/vocabularies/veecle/accept.txt`.
