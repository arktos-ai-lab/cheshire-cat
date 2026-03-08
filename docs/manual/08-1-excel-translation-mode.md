> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from felix-cat.com for the Felix revival project.

# 8.1. Translation Mode

Translation mode is the normal mode for translating Excel worksheets. This is the mode that you will spend the most time in.

There are a few things to keep in mind when translating in Excel. Firstly, everything is done at the cell level. Even if a cell contains multiple sentences, it’s currently not possible to translate at lower than the cell (or textbox) level. If you have a very long segment of text in a cell/textbox, you might want to consider copying and pasting that text into Word and translating it there.

Secondly, in order for edits you’ve made to cell to take effect, you first have to navigate out of that cell. So after you’ve translated a cell, first press Enter or Tab to go to a different cell, then go back to the translated cell, and press Alt + ↑ or the equivalent toolbar button/menu command to add that translation.

## 8.1.1. Look up a Cell

Select the cell you wish to look up. Then select *Felix ‣ Lookup* from the menu, or press Alt + L.

Note

Unlike in Word, formatting information is not preserved for Excel queries.

| Toolbar button |  |
| --- | --- |
| Menu command | Lookup |
| Keyboard shortcut | Alt + L |

Note

If a text box is selected, then the contents of the text box will be looked up.

## 8.1.2. Navigate through the Matches

Sometimes when you look up a sentence in your memory, Felix will have more than one suggestion.

The number of suggestions, and the number of the current suggestion, will be shown at the bottom right of the suggestion:

Fuzzy match view in Felix

In this example, the first of two suggestions (1/2) is being shown.

To view the next suggestion, click on Next in the Memory window, or from Word, select *Felix ‣ Next Translation* (or press Alt + N).

Tip

The score for the next translation (in this example 68%) is shown next to the Next link.

To view the previous suggestion, click on Prev in the Memory window, or from Word, select *Felix ‣ Previous Translation* (or press Alt + P).

Tip

The score for the previous translation (in this example 68%) is shown next to the Prev link.

Matches will cycle: If you press Next while on the last suggestion, the first suggestion will be shown; likewise, pressing Prev while on the first suggestion will show the last suggestion.

## 8.1.3. Register a Translation

While a query is shown in the Memory window (see *Look up a Cell*), and select *Felix ‣ Set* (or press Alt + ↑). The contents of the current cell are registered as the translation.

Tip

Select *Felix ‣ Set And Next* (or press Alt + S) to register the current translation, then automatically select and look up the next cell. This is equivalent to pressing Alt + ↑, then Alt + →.

### 8.1.3.1. Register translation and look up next cell

| Toolbar button |  |
| --- | --- |
| Menu command | Set And Next |
| Keyboard shortcut | Alt + S |

### 8.1.3.2. Register the current cell as the translation

| Toolbar button |  |
| --- | --- |
| Menu command | Set |
| Keyboard shortcut | Alt + ↑ |

## 8.1.4. Look up the Next Cell

Select *Felix ‣ Lookup Next* from the menu, or press ALT + →.

| Toolbar button |  |
| --- | --- |
| Menu command | Lookup Next |
| Keyboard shortcut | Alt + → |

## 8.1.5. Get a Translation from Memory

While a suggestion is being shown in the Memory window (see *Look up a Cell*, above), select *Felix ‣ Get* (or press Alt + ↓). The current cell text is replaced by the translation text.

If you edit the translation and wish to register your new translation, press CTRL + Enter, and then select *Felix ‣ S* (or press Alt + ↑). See *Register a Translation* for details.

Tip

Select *Felix ‣ Get And Next* (or press Alt + G) to get the current translation, then register this source/translation pair and automatically select and look up the next cell. This is equivalent to pressing Alt + ↓, Alt + ↑, then Alt + →.

### 8.1.5.1. Get the translation, and look up the next cell

| Toolbar button |  |
| --- | --- |
| Menu command | Get And Next |
| Keyboard shortcut | Alt + G |

### 8.1.5.2. Get the translation

| Toolbar button |  |
| --- | --- |
| Menu command | Get |
| Keyboard shortcut | Alt + ↓ |

## 8.1.6. Auto Translate

Sometimes, such as when translating a table or a list, you just know that most of the items are going to have 100% matches in your translation memory, and manually going through and checking each cell would be too tedious. If so, then you can use the Auto Translate function to look up every cell in the current selection, and automatically replace cells having a 100% match in the memory with their translations.

To use this function, select the range of text you wish to auto translate, then select *Felix ‣ Auto Translate*... from the menu, and select the auto translation type:

Selection
The current selection
Sheet
The current worksheet
All Workbooks
All open workbooks

Tip

If you hold down the Shift key while selecting one of the commands above, all cells that are not translated (because there was no 100% match) will be given a different background color. When you later translate those cells, the original background color will be restored. This lets you quickly spot and translate all the cells that were not translated by this function.

## 8.1.7. Register a Glossary

1.

Create a glossary in Excel consisting of two columns, with an optional third one; the first column containing the source segments, the second column containing the translation segments, and the third (optional) column containing context strings.

>

Tip

There can be other columns, and the columns do not have to be in any particular location. For instance, the sources might be located in column B, the translations in column C, and the contexts in column D, with other information in columns A and E. That doesn’t matter, as long as the two columns described here are present. If you do not include context, make sure that the third column is blank.

2.

Select the upper-left cell of the glossary table.

>

Excel Auto Register Glossary

3.

From the Felix menu, select Add Glossary.

4.

The glossary is automatically added to the MAIN Glossary window of Felix.

>

Tip

If either the source or translation column is empty, then that row is skipped. If three rows in a row are skipped, then Felix assumes that the end of the glossary has been reached, and stops adding entries.

## 8.1.8. Register a Memory

The procedure for registering a memory is similar to that for registering a glossary, the main difference being the lack of an optional extra column for context.

1.

Create a memory in Excel consisting of two columns; the first column containing the sources, and the second column their translations.

>

Tip

There can be other columns, and the columns do not have to be in any particular location. For instance, the sources might be located in column B, and the translations in column C, with other information in columns A and D. That doesn’t matter, as long as the two columns described here are present.

2.

Select the upper-left cell of the memory table.

3.
Select *Felix ‣ Add Memory* from the menu.

The memory is automatically added to the Memory window of Felix.
