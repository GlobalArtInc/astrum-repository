package org.globalart.astrum.repository.huggingface.internal.search;

import java.util.List;

import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;

/**
 * Search terms besides the repository id (the component name) and the commit (the version): the branches and tags
 * that resolved to the commit, and the paths of the files.
 */
@Component
@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceSearchMappings
    implements SearchMappings
{
  public static final String REVISION = "huggingface.revision";

  public static final String FILE = "huggingface.file";

  private static final List<SearchMapping> MAPPINGS = List.of(
      new SearchMapping(REVISION, "attributes.huggingface.revisions", "Hugging Face branch or tag", FORMAT_FIELD_1),
      new SearchMapping(FILE, "attributes.huggingface.file", "Hugging Face file path", FORMAT_FIELD_2, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
