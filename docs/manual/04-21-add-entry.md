> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from felix-cat.com for the Felix revival project.

# 4.2.1. Add an Entry

An entry, or translation unit (TU), is the basic unit of Felix translation memories (TMs) and glossaries. An entry consists of a source segment and its translation, as well as other optional information such as context, when the entry was created and by whom, and reliability.

1.

From the Edit menu, select Add Entry.

>

2.

The Add Entry dialog appears.

>

3.

Fill in the entry fields.

>

| Label | Description |
| --- | --- |
| SOURCE: | The source for the memory record (required) |
| TRANSLATION: | The translation for the memory record (required) |
| CONTEXT: | The context for the memory record (optional) |
| RELIABILITY: | The reliability score for the memory record (optional) |
| VALIDATED: | Whether the memory record is validated (optional) |
| CREATED: | The date the record was created (read-only) |
| MODIFIED: | The date the record was last modified (read-only) |
| EXTRA STRINGS [1]: | User-defined extra strings (optional ). Use the Edit Strings and Add String buttons to edit/add user extra strings. |

**Editor and Source tags**

The Source, Translation, and Context fields have Editor and Source tags. The Editor tags provide a WYSIWYG editor, enabling you to edit entries much like you would with a popular word-processing program. The Source tags, meanwhile, allow you to edit the actual HTML source code of each field, giving you greater control.

**Editor mode**

In Editor mode, a full range of text-formatting features is available for the source, translation, and context fields. In addition to common keyboard shortcuts such as Control-B for bold and Control-I for italic, you can set the font information for the current selection from the Font dialog. To call up the Font dialog, from the Format menu, select Font….

**Special symbols**

If you would like to insert a special symbol, select **Format** >> **Insert Symbol…**. The Insert Symbol dialog appears. Enter the HTML symbolic name or unicode number, without the leading ampersand (&) or trailing semicolon (;). For instance, entering “beta” will input ”β,” and entering ”#128” will input ”€.” The symbol is inserted at the current cursor position.

**Handling of newlines**

Because this is a dialog, the return key is handled in a special way. Pressing the return key by itself is the same clicking **OK**. To insert a pagraph marker (<P>), press Ctrl + Return. To insert a newline (<BR>), press Shift + Return.

4.

Click **OK** when you are finished entering the information.

5.

The entry is added to the memory.

>

| The extra strings are mainly useful when you import TMs from other formats. If a tag in another memory format isn’t used by Felix, it will be stored as an “extra string.” Then if you export your Felix memory back into that external format, the original tag information wil be preserved. |
| --- |
