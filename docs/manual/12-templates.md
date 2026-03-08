> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from felix-cat.com for the Felix revival project.

# 12. Customizing Felix Templates

This page describes how to use the Felix templates and style sheet to customize the way that Felix presents information.

## 12.1. Introduction

Felix uses templates to present most of its information: translation matches, concordance results, and search results. It also uses a CSS style sheet to control presentation.

You can modify these templates or CSS style sheet in order to customize how information is presented in Felix. One caveat: when you install a Felix upgrade, the templates and style sheet will be overwritten, so be sure to back up your modified files before upgrading.

The files are located in the following directory.

Vista:
`C:\Users\UserName\AppData\Local\Felix\html\ `
XP:
`C:\Documents and Settings\UserName\Local Settings\Application Data\Felix\html\ `

Where UserName is your account user name. The files in the “en” folder are English, and those in the “jp” folder are Japanese.

## 12.2. Syntax Primer

The syntax of the templating language is very simple. There are two types of constructs: variables and loops. If a variable has attributes, you can use the dot (”.”) operator to access them.

### 12.2.1. Variables

Variables are included like this: {$variable}. Assuming we have a variable “food”, it would look like this:

```
I love {$food}.
```

If the variable “food” is set to “eggy-weggies”, the result will be:

I love eggy-weggies.

If the variable has an attribute, you’d access it with the dot (”.”) operator: {$variable.attribute_name}. For example, in concordance results there is a records collection. Each record in that collection has several attributes. Assuming you have a record variable named “record”, you could thus do this:

```
The source is '{$record.source}'.
```

If the value of the “source” attribute is “long-ticks of toast”, the result will be:

```
The source is 'long-ticks of toast'.
```

### 12.2.2. Loops

Loops are used to present each of a collection of objects (such as matches or concordance results). Loops start with:
{% for varname in collection_name %}. They end with:
{% endfor %}.

For example, given a collection name “records”, you could access each record as follows:

```
The source values are:
{% for record in records %}{$record.source}
{% endfor %}
```

If there were two records, with source strings “source 1” and “source 2”, the output would be as follows:

```
The source values are:
source 1
source 2
```

#### 12.2.2.1. Special loop Variable

Within loops, a loop variable is also available, with the following attributes.

index
The item number in the loop (1, 2, ... n)
index0
The 0-based index
length
The number of items in the collection

Here’s an example of the loop variable in action:

```
The index values are
{% for r in records %}{$loop.index} {% endfor %}
```

If there were two records, the output would be as follows:

```
The index values are::
1 2
```

## 12.3. Templates

Below are the various templates used in Felix.

### 12.3.1. Match Templates

This section lists the variables available for match templates.

### 12.3.2. Single match view

These are the variables for files:

>
match_fuzzy.txt
match_perfect.txt
match_fuzzy_full.txt
match_perfect_full.txt

query_color
The text color for query segments
source_color
The text color for source segments
trans_color
The text color for translation segments
index
The match number (0-based)
query
The query segment
source
The source segment
trans
The translation segment
context
The context of the current record
created
The date the record was created
modified
The date the record was last modified
reliability
The reliability score of the record (number between 0 and 9)
validated
Whether the record has been validated
creator
The creator of the record
modified_by
The last user to modify the record
score
The match score
num
The index of this match (1-based)
total
The total number of matches
next_nav
The text for the Next/Previous links (if any)
mem
The name of the memory that this record belongs to
refcount
The reference count of the record
ref_count
alias for refcount

#### 12.3.2.1. matches_all.txt

These are the variables for file: matches_all.txt

query_color
The text color for query segments
source_color
The text color for source segments
trans_color
The text color for translation segments
records

The collection of matches for the current query. Each record object has the following attributes (accessed via the dot (”.”) operator, e.g. “{$record.query}”).

active
If this is the active record, then it will be “>> ”. Otherwise it will be empty.
query
The query segment
source
The source segment
trans
The translation segment
score
The match score
context
The context of the current record
created
The date the record was created
modified
The date the record was last modified
reliability
The reliability score of the record (number between 0 and 9)
validated
Whether the record has been validated
creator
The creator of the record
modified_by
The last user to modify the record
mem
The name of the memory that this record belongs to
refcount
The reference count of the record
ref_count
alias for refcount

### 12.3.3. Search Window Templates

These are the variables for search window template:

#### 12.3.3.1. search_matches.txt

results

The collection of matches for the current query. Each record object has the following attributes (accessed via the dot (”.”) operator, e.g. “{$record.query}”).

source
The source segment
trans
The translation segment
context
The context of the current record
created
The date the record was created
modified
The date the record was last modified
reliability
The reliability score of the record (number between 0 and 9)
validated
Whether the record has been validated
creator
The creator of the record
modified_by
The last user to modify the record
num
The index of the current match
num0
The 0-based index of the current match
mem
The name of the memory that this record belongs to
memory
Alias for mem
refcount
The reference count of the record
ref_count
alias for refcount

message
Any messages (e.g. feedback)
pagination
The pagination text (e.g. previous page/next page links)
page
The current page number in the results

### 12.3.4. Concordance/Glossary Match Templates

These are the variables for the concordance and glossary match templates:

>
mem_concordance.txt
mem_concordance0.txt
gloss_match.txt
gloss_match0.txt

records

The collection of matches for the current query. Each record object has the following attributes (accessed via the dot (”.”) operator, e.g. “{$record.query}”).

active
If this is the active record, then it will be “>> ”. Otherwise it will be empty. (Available for concordance view only)
source
The source segment
trans
The translation segment
context
The context of the current record
created
The date the record was created
modified
The date the record was last modified
reliability
The reliability score of the record (number between 0 and 9)
validated
Whether the record has been validated
creator
The creator of the record
modified_by
The last user to modify the record
mem
The name of the memory that this record belongs to
refcount
The reference count of the record
ref_count
alias for refcount

### 12.3.5. Other Templates

#### 12.3.5.1. item_info.txt

The “item_info.txt” template is for showing information about the selected memory or glossary in the Memory Manager dialog. The variables are as follows:

file_name
The name of the memory/glossary file
creator
The creator of the memory/glossary
field
The field that the memory/glossary belongs to
created_on
Date created
source_language
The source language of the memory/glossary
target_language
The target language of the memory/glossary
client
The client
mem_size
Number of records
file_size
The size of the file
reliability
Average reliability of records in the memory/glossary
validated
Proportion of records that are validated
locked
Whether the memory/glossary is locked (records can’t be added or deleted)

#### 12.3.5.2. next_nav.txt

prev_score
The score of the previous match
next_score
The score of the next match
