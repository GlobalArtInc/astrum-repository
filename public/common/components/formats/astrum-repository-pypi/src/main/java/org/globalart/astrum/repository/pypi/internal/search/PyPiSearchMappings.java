
package org.globalart.astrum.repository.pypi.internal.search;

import java.util.List;

import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_3;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_4;

@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = ImmutableList.of(
      new SearchMapping("pypi.classifiers", "assets.attributes.pypi.classifiers", "PyPI classifiers",
          FORMAT_FIELD_1, false),
      new SearchMapping("pypi.description", "assets.attributes.pypi.description", "PyPI description",
          FORMAT_FIELD_2, false),
      new SearchMapping("pypi.keywords", "assets.attributes.pypi.keywords", "PyPI keywords",
          FORMAT_FIELD_3, false),
      new SearchMapping("pypi.summary", "assets.attributes.pypi.summary", "PyPI summary",
          FORMAT_FIELD_4, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
