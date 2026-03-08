> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from felix-cat.com for the Felix revival project.

# 11.1.1.2.6. SearchResults

This represents a collection of *SearchResult* objects (translation memories and glossaries are both represented by this object).

## 11.1.1.2.6.1. Properties

| Name | Type | Access | Description |
| --- | --- | --- | --- |
| Count | Int | Get | The number of items in the collection |

## 11.1.1.2.6.2. Methods

| Name | Arguments | Description |
| --- | --- | --- |
| Item | Int *n* | Gets the *SearchResult* at index *n* |

## 11.1.1.2.6.3. Example

Note that since this is a collection, you can iterate over it in Visual Basic and other languages.

From Visual Basic:

```vba
Dim match As Object
For Each match In Felix.App2.CurrentMatches
    Debug.Print match.MemoryName
Next match
```
