package org.globalart.astrum.repository.huggingface.internal;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class HuggingFaceExceptionTest
{
  @Test
  void survivesTheNegativeCache() {
    HuggingFaceException original = new HuggingFaceException(404, "RevisionNotFound", "Invalid rev id: v9: beta");
    HuggingFaceException decoded = HuggingFaceException.decode(404, original.encode());
    assertThat(decoded.getStatus(), is(404));
    assertThat(decoded.getErrorCode(), is("RevisionNotFound"));
    assertThat(decoded.getMessage(), is("Invalid rev id: v9: beta"));

    HuggingFaceException plain = HuggingFaceException.decode(404, "acme/model at main is not available");
    assertThat(plain.getErrorCode(), is(nullValue()));
    assertThat(plain.getMessage(), is("acme/model at main is not available"));
  }

  @Test
  void namesTheCommitOfMissingFiles() {
    HuggingFaceException missing = HuggingFaceException.entryNotFound("abc", "adapter_config.json");
    assertThat(missing.getStatus(), is(404));
    assertThat(missing.getErrorCode(), is("EntryNotFound"));
    assertThat(missing.getCommit(), is("abc"));
  }
}
