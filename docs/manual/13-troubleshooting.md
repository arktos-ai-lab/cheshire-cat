> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 13. Troubleshooting Felix

## 13.1. Installation/Uninstallation

### 13.1.1. Installation fails with an “Access Denied” error

Make sure that all Microsoft Office programs are closed before installing. This includes Microsoft Word, Excel, and PowerPoint.

### 13.1.2. Some items could not be removed during uninstallation

Those items may have been in use by another program. Check the files or folders, and delete them if you no longer need them.

The places to check are as follows:

```
{Program Files}/Assistant Suite/Felix
{Local App Data}/Felix
```

## 13.2. Felix

### 13.2.1. I looked up white text, and I can’t see it in Felix

Change the background color of the window so that there is a contrast with the text. (Select *Format ‣ Background Color*... from the menu.)

## 13.3. Word

### 13.3.1. The toolbar and menu don’t show up

Word may have disabled the Felix add-in. See this blog post for details.

### 13.3.2. Some keyboard shortcuts don’t work

If only the two-key shortcuts (like Alt + M, S) don’t work, make sure that the IME is not turned on. If the IME is turned on, you must use the menu commands for these.

### 13.3.3. The translation history feature does not reflect my edits correctly

It might be that your document is too complex, or your edits have been too complex. If the translation history does not parse your document correctly, please disable it and use Review mode normally.

## 13.4. Excel

### 13.4.1. The toolbar and menu don’t show up

Excel may have disabled the Felix add-in. See this blog post for details.

### 13.4.2. When I set a translation, the old text is added instead of my translation

Press CTRL + Enter to finalize the text in a cell before adding your translation.

## 13.5. PowerPoint

### 13.5.1. The toolbar and menu don’t show up

PowerPoint may have disabled the Felix add-in. See this blog post for details.

### 13.5.2. PowerPoint skips text boxes when I set a translation and go to the next one

PowerPoint looks for text boxes from left to right, top to bottom. If it skips over a text box, put the cursor in that text box manually and select *Felix ‣ Look Up Next*.

### 13.5.3. I get a COM error when I try to look up text

Make sure that the text isn’t in a grouped shape. If it is, right click it and select Ungroup, then try translating it again.
