
package org.globalart.astrum.repository.maven.internal;

import java.io.ByteArrayInputStream;

import org.apache.maven.artifact.repository.metadata.Metadata;
import org.apache.maven.model.Model;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.nullValue;

public class MavenModelsTest

{
  String notXml = "not xml";

  @Test
  public void testReadModel_emptyInputStreamIsNull() throws Exception {
    Model model = MavenModels.readModel(new ByteArrayInputStream(new byte[0]));
    assertThat(model, nullValue());
  }

  @Test
  public void testReadModel_NotXmlIsNull() throws Exception {
    Model model = MavenModels.readModel(new ByteArrayInputStream(notXml.getBytes()));
    assertThat(model, nullValue());
  }

  @Test
  public void testReadModel_WithoutClosingTagsIsNull() throws Exception {
    Metadata metadata = MavenModels.readMetadata(
        getClass().getResourceAsStream("/org/globalart/astrum/repository/maven/metadataWithoutClosingTags.xml"));
    assertThat(metadata, nullValue());
  }
}
