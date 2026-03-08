> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from felix-cat.com for the Felix revival project.

# 11.1.1.2.9. MemoryWindow

The **MemoryWindow** represents the Felix memory or glossary window. You can use it to change the position and focus of the memory and glossary windows.

## 11.1.1.2.9.1. Properties

| Name | Type | Access | Description |
| --- | --- | --- | --- |
| Height | Int | Get/Set | The window height |
| Left | Int | Get/Set | The left edge of the window |
| Top | Int | Get/Set | The top edge of the window |
| Width | Int | Get/Set | The window width |

## 11.1.1.2.9.2. Methods

| Name | Arguments | Description |
| --- | --- | --- |
| Raise | *none* | Raises the Felix window so that it’s visible |

## 11.1.1.2.9.3. Example

Raise the Felix memory window:

```vba
Dim felix As Object
Set felix = CreateObject("Felix.App")

Dim mem As Object
Set mem = felix.App2.MemoryWindow

call mem.Raise
```
