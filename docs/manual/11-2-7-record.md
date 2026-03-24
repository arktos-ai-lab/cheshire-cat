> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 11.1.1.2.7. Record

The **Record** object represents a Felix translation memory or glossary. It is normally retrieved from a *Records* collection.

## 11.1.1.2.7.1. Properties

| Name | Type | Access | Description |
| --- | --- | --- | --- |
| Context | String | Get/Set | The context |
| CreatedBy | String | Get/Set | Who created the record |
| DateCreated | Date | Get/Set | The date that the record was created |
| Id | Int | Get/Set | The record’s ID in the TM/glossary |
| LastModified | Date | Get/Set | When the record was last modified |
| ModifiedBy | String | Get/Set | Who last modified the record |
| PlainContext | String | Get | Plain text context (no tags) |
| PlainSource | String | Get | Plain text source (no tags) |
| PlainTrans | String | Get | Plain text context (no tags) |
| RefCount | Int | Get/Set | The number of times that the record has been referenced (used) |
| Reliability | Int | Get/Set | A number between 0 and 9 representing the reliability (higher is more reliable) |
| Source | String | Get/Set | The source |
| Trans | String | Get/Set | The translation |
| Validated | Boolean | Get/Set | Whether the record has been validated |

## 11.1.1.2.7.2. Methods

None

## 11.1.1.2.7.3. Example

Here is some code to validate every record that was created by Ryan:

```vba
Sub ValidateIfByRyan() ' :)

    Dim felix As Object
    Set felix = CreateObject("Felix.App")
    felix.Visible = True

    Call AddMemoryEntries

    ' Create the gloss and mem objects
    Dim mem As Object
    Set mem = felix.App2.Memories.Item(1)

    ' Loop through the records
    Dim record As Object
    For Each record In mem.records
        If record.CreatedBy = "Ryan" Then
            Debug.Print "Created by Ryan: validating..."
            record.Validated = True
        End If
    Next record

End Sub
```
