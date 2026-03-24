> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 11.1.1.2.8. Records

This represents a collection of *Record* objects (translation memories and glossaries are both represented by this object).

## 11.1.1.2.8.1. Properties

| Name | Type | Access | Description |
| --- | --- | --- | --- |
| Count | Int | Get | The number of items in the collection |

## 11.1.1.2.8.2. Methods

| Name | Arguments | Description |
| --- | --- | --- |
| Item | Int *n* | Gets the *Record* at index *n* (1`based) |

## 11.1.1.2.8.3. Example

Note that since this is a collection, you can iterate over it in Visual Basic and other languages.

From Visual Basic:

```vba
Dim record As Object
For Each record In mem.Records
    Debug.Print record.Source
Next record
```
