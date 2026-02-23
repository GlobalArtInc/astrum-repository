
package org.globalart.astrum.repository.rest.sql;

/**
 * Indicates how the column should be queried. That is, does it use normal operators (i.e. '=', 'LIKE' and 'IN')
 * or is it a Full text query (i.e. queried using &lt;column&gt; @@ to_tsvector('simple', 'foo'))
 *
 * @see org.globalart.astrum.repository.rest.SearchFieldSupport
 */
public enum TextualQueryType
{
  /**
   * This means default text column querying approach i.e. using '=', 'IN'
   */
  DEFAULT_TEXT_QUERY,

  /**
   * For postgres: this means querying of a full text column using postgres's tsquery()
   * For h2: this means querying an ARRAY type column
   */
  FULL_TEXT_SEARCH_QUERY
}
