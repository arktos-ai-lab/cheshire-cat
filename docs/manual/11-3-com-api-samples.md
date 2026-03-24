> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 11.1.1.3. Felix COM API Code Samples

This page contains code samples to illustrate the use of the COM API.

## 11.1.1.3.1. Look up a Query

This will look up a query in the Felix window:

```vba
Dim felix As Object
Set felix = CreateObject("Felix.App")
felix.LookUp "My query"
```

## 11.1.1.3.2. Get Translation Info

This will print out information about the match suggested for the current query:

```vba
Dim felix As Object
Set felix = CreateObject("Felix.App")

Debug.Print "Number of matches:" & felix.NumMatches
Debug.Print "Score: " & felix.Score
Debug.Print "Translation: " & felix.trans
```

## 11.1.1.3.3. Set the Translation for a Query

Here, we will look up a sentence, and then provide a translation:

```vba
Dim felix As Object
Set felix = CreateObject("Felix.App")
felix.LookUp "My query"
felix.Trans = "This is the translation"
```

## 11.1.1.3.4. Merge Two TMs

Here’s a slightly more involved example. Here, we’ll load two TMs, merge them into a single TM, and then save it with a new name:

```vba
Dim felix As Object
Set felix = CreateObject("Felix.App")

' load the two mems
Dim mem1, mem2 As Object
Set mem1 = felix.App2.Memories.Load("c:\mem1.ftm")
Set mem2 = felix.App2.Memories.Load("c:\mem2.ftm")

' Now add all the records in mem2 to mem1
For Each record In mem2.Records
    mem1.AddRecord record
Next record

' Finally, save the mem with a new name
mem1.SaveAs "c:\merged.ftm"
```
