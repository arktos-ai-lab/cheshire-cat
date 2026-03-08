> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from felix-cat.com for the Felix revival project.

# 11.1.1.2.4. Memory

The **Memory** object represents a Felix translation memory or glossary. It is normally retrieved from a *Memories* collection.

## 11.1.1.2.4.1. Properties

| Name | Type | Access | Description |
| --- | --- | --- | --- |
| Client | String | Get/Set | The client that this TM/glossary is for |
| CreatedOn | Date | Get/Set | The date and time that this TM/glossary was created |
| Creator | String | Get/Set | The creator of the TM/glossary |
| Field | String | Get/Set | The field of the TM/glossary (e.g. “Electronics” or “Education”) |
| IsLocked | Boolean | Get/Set | Whether the TM/glossary is locked. It is not possible add/delete records to/from locked Memory objects. |
| IsMemory | Boolean | Get/Set | True if the object is a TM, False if it is a glossary |
| ModifiedBy | String | Get/Set | The last user to modify the TM/glossary |
| ModifiedOn | Date | Get/Set | The date and time that this TM/glossary was last modified |
| Records | Object | Get | The *Records* collection representing the records in the TM/glossary |
| SourceLanguage | String | Get/Set | The source language of the TM/glossary |
| TargetLanguage | String | Get/Set | The target language of the TM/glossary |

## 11.1.1.2.4.2. Methods

| Name | Arguments | Description |
| --- | --- | --- |
| AddRecord | Object *Record* | Adds a *Record* object to the TM/glossary |
| GetSize | *none* | Returns the size of the TM/glossary |
| RemoveRecord | Object *Record* | Removes the specified *Record* object from the TM/glossary |
| Save | *none* | Saves the TM/glossary at the current location |
| SaveAs | String *Location* | Saves the TM/glossary at the specified location |

## 11.1.1.2.4.3. Example

Here’s some sample code to print out information about the active memory:

```vba
Sub PrintActiveMemoryInfo()

    Dim felix As Object
    Set felix = CreateObject("Felix.App")
    felix.Visible = True

    ' Create the mem object
    Dim mem As Object
    Set mem = felix.App2.ActiveMemory

    ' Print out info
    Debug.Print mem.Creator
    Debug.Print mem.CreatedOn
    Debug.Print mem.Client
    Debug.Print mem.Field

End Sub
```
