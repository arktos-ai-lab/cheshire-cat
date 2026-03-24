> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 14. What’s New in Felix

**Version 1.7.3**

>

- Added option to save login credentials for remote TMs and glossaries.

- **Bug Fix**: Fixed bug where glossary entries would be registered to the wrong glossary.

**Version 1.7.2.1**

>

- **Bug Fix**: Fixed bug when number placement and glossary highlighting were used together.

**Version 1.7.2**

>

- Improved stability with Office 2013.

- Option to highlight glossary matches. See *View Page* for details.

- Glossary/concordance view no longer scrolls to top when an entry is deleted.

- Windows 2000 is no longer supported.

- Windows XP is no longer supported.

**Version 1.7.1.3**

>

- **Bug Fix**: Pressing the Control key in PowerPoint sometimes activated Felix unintentionally.

- Non-ASCII text was sometimes garbled in Register Glossary dialog box.

**Version 1.7.1.2**

>

-

**Bug Fix**: Some “Delete” links did not work correctly.

-

Minor improvements to Rule Manager.

>

The Rule Manager no longer imports duplicate entries. You can export and import rules without creating duplicates.

**Version 1.7.1.1**

>

-

**Bug Fix**: Corrupt TMX files could cause Felix to crash.

-

Felix no longer marks imported TMX files as changed automatically.

>

This would cause Felix to prompt the user to save the TMX file, even if it had not been changed.

-

**Bug Fix**: Failed to mark TMs as changed when merged with another TM.

**Version 1.7.1**

>

-

Rule placement option

>

There is now an option to automatically insert rule-based substitutions into your translations. See *Rule-based Placement* for details.

-

**Bug Fix**: Preferences did not load TMs/glossaries

>

After switching to the new preferences format, Felix failed to load TMs and glossaries when a preferences file was loaded.

-

**Bug Fix**: Stopped asking about whether to merge TMs/glossaries

>

After you selected “Don’t ask me again” in the merge dialog, you would not be queried again about merging TMs/glossaries, even if you selected this option in the properties.

-

New toolbar button for Memory Manager

-

**Bug Fix**: Felix for MS Word did not correctly parse non-breaking hyphen characters.

-

Improved segmentation in MS Word.

**Version 1.7**

>

-

Glossary placement option

>

There is now an option to place glossary matches, similar to the current number placement feature. See *Glossary Placement* for details.

**Version 1.6.9.3**

>

-

**Bug Fix**: Match view in review mode was sometimes reversed.

>

In review mode, the source and translation would sometimes appear reversed in the match view of the Felix window.

**Version 1.6.9.2**

>

-

**Bug Fix**: Could not add two translations or glossary entries in a row from the dialog.

>

If the Add Entry dialog was used to add a translation or glossary entry two times in a row, the first entry would be replaced by the second.

-

**Bug Fix**: Formatting penalty was calculated incorrectly.

>

There was a problem with the way formatting penalties were calculated that created penalties even when there were no formatting differencies.

**Version 1.6.9.1**

>

-

**Bug Fix**: Could not browse TMs and glossaries in Manger window past first page

>

A regression prevented users from browsing past the first page in the Browse view of the Manager window.

**Version 1.6.9**

>

-

**Bug Fix**: Rare crashes in Search window

>

Under rare circumstances, an invalid query syntax in the Search window could crash Felix.

-

**Bug Fix**: CTRL + ALT + F9 keyboard shortcut does not work

>

The CTRL + ALT + F9 keyboard shortcut for disabling/enabling keyboard shorcuts in MS Office did not work.

-

Support times in searches

>

Times are now supported in date-related searches. For example, the date created can now be specified with the date and time, rather than just the date.

Date-times are specified in the following format:

```
YYYY/MM/DD [HH:MM:SS:UU]
```

-

Active setting for Manager Window

>

You can now specify translation memories and glossaries as active or inactive. An inactive TM/glossary can be used for concordance and searching, but is not used for memory and glossary searches.

**Version 1.6.8.1**

>

-

Choose glossaries in QC settings

>

You can now choose which glossaries you want to use for QC on your TMs.

-

Parentheses after periods selected in lookup

>

When users look up sentences in Word and PowerPoint, a parenthesis following a period is now correctly included in the selection.

**Version 1.6.8**

>

-

Login option for Memory Serves

>

Users can now log into Memory Serves with a username and password.

-

New help system

>

Felix has a new help system. In addition to improved and expanded help, the new manual includes a search page and index.

**Version 1.6.7**

>

-

“One translation per source” option

>

There is now an option to only allow one translation per source segment. By default, Felix allows any number of translations for the same source segment.

When this feature is enabled, if you add a new translation for an existing source segment in the TM, the existing record will be replaced.

To enable this feature, go to **Tools** > **Preferences**, then click on the **Memory tab**, and select “One translation per source segment.”

**Note**: You should set this option before loading any TMs. If you change it after a TM is loaded, you could get unexpected results.

-

PowerPoint Felix interface made consistent with Word and Excel interfaces

>

The Felix interface for PowerPoint now matches those for Word and Excel, with a translation and review mode.

-

“Correct Translation” keyboard shortcut for Excel

>

There is now a “Correct Translation” keyboard shortcut option for Excel. To activate this option, go to the Felix menu in Excel, and select Preferences. Then select **Keyboard Shortcuts**,　and assign a keyboard shortcut for “Correct Translation”. The default is CTRL+ALT+UP.

**Version 1.6.6**

-

“Remove All” feature for Memory Manager

>

You can now remove all TMs/glossaries in the Memory Manager window with a single click.

-

**Bug Fix**: Correcting translation creates new record

>

There was a bug where in some cases, “correcting” a translation actually added a new entry.

-

**Bug Fix**: Long “add entry” entries get broken into multiple lines

>

In the “Add Entry” dialog, long entries would get broken into multiple lines.

-

Memory Manager: Give option to show old manager

>

Some users with older versions of Internet Explorer (typically corporate mandated) had problems using the features of the new Memory Manager window. There is now an option to use the old Memory Manager dialog: from the Preferences dialog (Tools &gt;&gt; Preferences), select the “Use old memory manager” checkbox.

-

**Bug Fix**: Broken css/javascript on Japanese Memory Manager window

>

On the Japanese version of the Memory Manager window, pages were not displayed properly due to broken CSS and JavaScript links.

-

Allow unlimited number of files in loaded history

>

You can now keep an unlimited number of files in the loaded history (used in the “Load previous memories/glossaries on next startup” option). The previous limit was 15 items each.
