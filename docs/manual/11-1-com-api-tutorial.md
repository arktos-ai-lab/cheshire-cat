> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from cheshire-cat.com for the Felix revival project.

# 11.1.1.1. Felix COM API Tutorial

This tutorial will show you how to get started scripting Felix via the COM API. We’ll be using Microsoft Word’s VBA as the tutorial environment.

## 11.1.1.1.1. Start Word and Open Visual Basic

Start MS Word, and open the Visual Basic for Applications editor. You can do this by pressing ALT+F11, or from the Developer tab, clicking **Visual Basic**.

## 11.1.1.1.2. Create a New Module

In the left pane, there should be a “Normal” node. Expand this, and right click on the “Modules” folder.

From the context menu, select **Insert** >> **Module**. This is where we’ll be putting in our code.

## 11.1.1.1.3. Write Macro to Launch Felix

Now, let’s write a couple of macro routines.

First, add this routine to the module you just created. It launches Felix and makes it visible:

```vba
Sub LaunchFelix()

    Dim felix As Object
    Set felix = CreateObject("Felix.App")
    felix.Visible = True

End Sub
```

Now, put your cursor somewhere in the macro, and press F5 to run it. Felix should now be launched and visible, if it wasn’t already.

## 11.1.1.1.4. Add Some Translations to the TM

Next, we’ll add a couple of translation entries.

Add the following macro to your module, and run it in the same way:

```vba
Sub AddMemoryEntries()

    Dim felix As Object
    Set felix = CreateObject("Felix.App")
    felix.Visible = True

    ' add some entries
    felix.AddMemoryEntry "source 1", "trans 1", "context 1"
    felix.AddMemoryEntry "source 2", "trans 2", "context 2"

End Sub
```

Check the Felix window. There should be two entries in the TM.

## 11.1.1.1.5. Add a Glossary Entry

Next, add a glossary entry. The procedure is very similar to adding TM entries:

```vba
Sub AddGlossaryEntries()

    Dim felix As Object
    Set felix = CreateObject("Felix.App")
    felix.Visible = True

    ' add some entries
    felix.AddGlossaryEntry "gloss 1", "abc", "context 1"

End Sub
```

You should now have one entry in your glossary.

## 11.1.1.1.6. Save the TM and Glossary

Finally, we’ll save the TM and glossary we created. Add and run this macro:

```vba
Sub SaveMemoryAndGlossary()

    Dim felix As Object
    Set felix = CreateObject("Felix.App")
    felix.Visible = True

    ' Create the gloss and mem objects
    Dim mem, gloss As Object
    Set mem = felix.App2.Memories.Item(1)
    Set gloss = felix.App2.Glossaries.Item(1)

    ' Save the TM and glossary
    mem.SaveAs "c:\my_tm.ftm"
    gloss.SaveAs "c:\my_gloss.fgloss"

End Sub
```

Check your C drive; you should find the TM “my_tm.ftm”, and the glossary “my_gloss.fgloss”.

See the *Felix COM API Code Samples* for more sample code, and *Felix COM API Objects* for a list of all the available objects.
