> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 7.4. Translation History

Translation history is a new feature in Felix version 1.4. Using a translation history, you can make edits to your translation, and then reflect those edits in your translation memory with a single operation.

Note

This feature is still in beta. It is not yet stable in all circumstances.

To use the Translation History feature, enable it from the preferences, on the Translation History tab. See *Setting Preferences* for details.

When you use the Translation History feature, Felix creates a ”.fhist” file alongside the document you’re translating. For example, if your document is “test.doc”, Felix will create a history file alongside it named “test.doc.fhist”. The history file contains a record of all the source sentences you’ve translated, their translations, and where they’re located in the document.

When you edit your translation, you can then go into Review mode (see *Review Mode*), then select *Felix ‣ Reflect Trans Edits*, and all your edits will be reflected in your translation memory automatically.

If you create a new memory in Felix, then reflecting your edits will create a TM with just the translations from your document.
