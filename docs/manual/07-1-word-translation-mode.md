> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 7.1. Translation Mode

Translation Mode is probably where you will spend most of your time with Felix. This mode is for translating documents, as opposed to Review Mode (see *Review Mode*), which is for reviewing translations.

Each of the Translation Mode functions is described below.

Note

In Word 2007 and 2010, the menu and toolbar appear on the Add-Ins tab.

## 7.1.1. Look up a Sentence

To have the suite automatically select the next sentence to look up, place the cursor in front of the sentence, and select *Felix ‣ Look up Next Setence* (or press ALT + →). The Felix interface will skip any whitespace before the next sentence, and select all the text up to the next tab, line break, or end-of-sentence marker (e.g. period). The Felix interface will then send the sentence to Felix for lookup.

Tip

You can configure which characters to count as the end of a segment. See *Setting Preferences* for details.

Most text formatting information (e.g. bold, italic, font, text color) is preserved in the query. Some formatting information, however, such as highlight color and centered text, is not preserved.

### 7.1.1.1. Extend the lookup segment

If you would like to extend the lookup segment beyond the current selection, then with the current lookup scope selected, select *Felix ‣ Extend Lookup Sentence* (or press Ctrl + →).

### 7.1.1.2. Select to the next period

To select all text to the next end-of-sentence marker (e.g. full stop or question mark), press Alt + . (period). This will select all the text up to the marker, regardless of intervening carriage returns or the like. This is handy when a sentence you need to translate has been formatted using hard breaks, or when you’ve copied and pasted text from a PDF file.

### 7.1.1.3. Select your own segment

You can also select any text as the lookup segment. Simply select a segment of text, then select *Felix ‣ Look up Current Selection* (or press ALT + L).

## 7.1.2. Navigate through the Matches

Sometimes when you look up a sentence in your memory, Felix will have more than one suggestion. The number of suggestions, and the number of the current suggestion, will be shown at the bottom right of the suggestion:

Fuzzy match view in Felix

In this example, the first of two suggestions (1/2) is being shown.

To view the next suggestion, click on Next in the Memory window, or from Word, select *Felix ‣ Next Translation* (or press Alt + N).

Tip

The score for the next translation (in this example 68%) is shown next to the Next link.

To view the previous suggestion, click on Prev in the Memory window, or from Word, select *Felix ‣ Previous Translation* (or press Alt + P).

Tip

The score for the previous translation (in this example 68%) is shown next to the Prev link.

Matches will cycle: If you press Next while on the last suggestion, the first suggestion will be shown; likewise, pressing Prev while on the first suggestion will show the last suggestion.

## 7.1.3. Register a Translation

While a query is shown in the Memory window (see *Look up a Sentence*), place the cursor at the end of your translation, and select *Felix ‣ Register Current Translation* (or press Alt + ↑). The segment from the start of the original sentence to the current cursor position is registered as the translation.

To manually select the segment to register as the translation, select a segment of text, then select *Felix ‣ Register Current Translation* (or press Alt + ↑). The current selection is registered as the translation.

Tip

Select *Felix ‣ Set And Next* (or press Alt + S) to register the current translation, then automatically select and look up the next sentence. This is equivalent to pressing Alt + ↑, then Alt + →.

## 7.1.4. Add a Glossary Entry

While you are translating, you may want to add terms to your glossary. You can do this easily from inside Word. Simply select the source text of your glossary term, and select *Felix ‣ Add to Glossary*. Alternatively, you can press ALT + M, G.

The Add to Glossary dialog box appears. Type in a translation, and click Add. The entry is added to the Felix glossary.

See *Add Glossary Entries from a Translation* for details.

## 7.1.5. Get a Translation from Memory

While a suggestion is being shown in the Memory window (see *Look up a Sentence*, above), select *Felix ‣ Get Current Translation* (or press Alt + ↓). The current selection is replaced by the suggestion.

If you edit the translation and wish to register your new translation, select *Felix ‣ Register Current Translation* (or press Alt + ↑). See *Register a Translation* for details.

Tip

Select *Felix ‣ Get And Next* (or press Alt + G) to get the current translation, then register this source/translation pair and automatically select and look up the next sentence. This is equivalent to pressing Alt + ↓, Alt + ↑, then Alt + →.

## 7.1.6. Delete a Translation

While a suggestion is being shown in the Memory window (see *Look up a Sentence*, above), select *Felix ‣ Delete Translation* (or press Alt + D), or in the Felix window, press the Delete link.

A message box will appear, asking if you really want to delete this translation. To delete the translation, click Yes. The currently displayed entry is deleted. If you change your mind, click No . The entry will not be deleted.

## 7.1.7. Get a Glossary Entry

Every time you look up a sentence in the memory, the sentence is also searched for matches in all open glossaries. Any matches are displayed in the corresponding Glossary window. Using the glossary for technical terms helps you keep your translations consistent, saves you typing, and saves time looking up paper or electronic reference materials.

Glossary Search Results View

As you are typing, you can retrieve any matches shown in the Main Glossary window via hot keys. Select *Felix ‣ Get Glossary Entry ‣ Entry {N}* (or press ALT + {N}), where {N} is a number between 0 and 9, to retrieve that glossary entry. If the number of the entry you wish to retrieve is higher than 9, then press ALT + M, H (note: the keyboard shortcut will not function if the IME is turned on). An input box appears; enter the entry number and press Return to retrieve the corresponding entry.

## 7.1.8. Search for Concordance

Oftentimes while you translate, you will wonder how you handled a particular term in previous translations. This is what the Concordance feature is for. Select the term you wish to investigate, then select *Felix ‣ Find Concordance*, or press Alt + M, C (note: this will not function if the IME is turned on).

Any entries in the memory containing that term in the Source field appear in the Memory window:
Concordance Results View

## 7.1.9. Save the Memory

A prudent backup strategy will save you future headaches in any computing situation, and Felix is no exception. In order to facilitate saving the memory and glossaries, you can perform this from Word via a keyboard shortcut: Just press Alt + M, S to save your current memory and glossaries (note: this will not function if the IME is turned on). You can also do this from the menu, by selecting *Felix ‣ Save Memory*.

## 7.1.10. Auto Translate the Current Selection

Sometimes, such as when translating a table or a list, you just know that most of the items are going to have 100% matches in your translation memory, and manually going through and checking each entry would be too tedious. If so, then you can use the Auto Translate function to look up every sentence/segment in the current selection, and automatically replace items having a 100% match in the memory with their translations.

To use this function, select the range of text you wish to auto translate, then select *Felix ‣ Auto Translate Selection* (or press ALT + M, A – note that this will not function if the IME is turned on).

## 7.1.11. Auto Translate until the Next Fuzzy Match

You can also auto translate from the current cursor position. Felix will automatically translate each sentence, until it reaches a sentence without a perfect match in the memory.

To use this function, select *Felix ‣ Translate to Fuzzy* (or press ALT + Z).
