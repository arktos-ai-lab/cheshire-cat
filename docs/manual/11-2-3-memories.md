> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 11.1.1.2.3. Memories

This represents a collection of *Memory* objects (translation memories and glossaries are both represented by this object).

## 11.1.1.2.3.1. Properties

| Name | Type | Access | Description |
| --- | --- | --- | --- |
| Count | Int | Get | The number of items in the collection |

## 11.1.1.2.3.2. Methods

| Name | Arguments | Description |
| --- | --- | --- |
| Item | Int *n* | Gets the *Memory* at index *n* |
| Add | *none* | Adds a *Memory* to the collection and returns it |
| Load | String *Location* | Loads a *Memory* at location *Location*, and returns it |
| Clear | *none* | Empties the collection |

## 11.1.1.2.3.3. Example

Note that since this is a collection, you can iterate over it in Visual Basic and other languages.

From Visual Basic:

```vba
Dim mem As Object
For Each mem In Felix.App2.Memories
    Debug.Print mem.CreatedOn
Next mem
```
