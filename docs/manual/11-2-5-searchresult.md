> **Note:** Documentation for Felix 1.7.1.1 (original). Preserved from felix-cat.com for the Felix revival project.

# 11.1.1.2.5. SearchResult

The **SearchResult** object represents a Felix translation memory or glossary. It is normally retrieved from a *SearchResults* collection.

A **SearchResult** object holds the matching TM/glossary record, the match score, and the name of the TM/glossary that the record is from.

## 11.1.1.2.5.1. Properties

| Name | Type | Access | Description |
| --- | --- | --- | --- |
| Record | Object | Get/Set | The *Record* object for this match. |
| Score | Double | Get | The match score |
| MemoryName | String | Get | The name of the TM/glossary that the record is from. |

## 11.1.1.2.5.2. Methods

None
