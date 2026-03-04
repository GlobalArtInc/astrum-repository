
package org.globalart.astrum.selector;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.selector.LeadingSlashRegexTransformer.trimLeadingSlashes;

public class LeadingSlashRegexTransformerTest
{
  @Test
  public void expectedLeadingSlashTransformations() {
    assertThat(trimLeadingSlashes(""), is(""));
    assertThat(trimLeadingSlashes("/"), is(""));
    assertThat(trimLeadingSlashes(".*"), is(".*"));
    assertThat(trimLeadingSlashes("/.*"), is(".*"));
    assertThat(trimLeadingSlashes(".*/"), is(".*(^|/)"));
    assertThat(trimLeadingSlashes("/.*/"), is(".*/"));
    assertThat(trimLeadingSlashes(".?/foo/"), is(".?(^|/)foo/"));
    assertThat(trimLeadingSlashes("/+foo/"), is("(^|/)+foo/"));
    assertThat(trimLeadingSlashes("/com|/org"), is("com|org"));
    assertThat(trimLeadingSlashes("(?!.*/struts/.*).*/apache/.*"), is("(?!.*(^|/)struts/.*).*(^|/)apache/.*"));
    assertThat(trimLeadingSlashes("((([a-z[A-Z]]/)?([0-9]?/)*)|.*/?)/foo"),
        is("((([a-z[A-Z]]/)?([0-9]?(^|/))*)|.*/?)foo"));
  }
}
