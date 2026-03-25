
package org.globalart.astrum.repository.maven.internal.search;

import java.util.List;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import com.google.common.collect.ImmutableList;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_3;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_4;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAME;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAMESPACE;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.7
 */
@Component
@Qualifier("maven2")
public class MavenSearchMappings
    implements SearchMappings
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String GAVEC = "gavec";

  private static final List<SearchMapping> MAPPINGS = ImmutableList.of(
      new SearchMapping("maven.groupId", "attributes.maven2.groupId", "Maven groupId", NAMESPACE),
      new SearchMapping("maven.artifactId", "attributes.maven2.artifactId", "Maven artifactId", NAME),
      new SearchMapping("maven.baseVersion", "attributes.maven2.baseVersion", "Maven base version", FORMAT_FIELD_1),
      new SearchMapping("maven.extension", "assets.attributes.maven2.extension", "Maven extension of component's asset",
          FORMAT_FIELD_2),
      new SearchMapping("maven.classifier", "assets.attributes.maven2.classifier",
          "Maven classifier of component's asset", FORMAT_FIELD_3),
      new SearchMapping(GAVEC, GAVEC, "Group asset version extension classifier", FORMAT_FIELD_4));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
