
package org.globalart.astrum.content.maven.internal.search.table;

import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;
import org.globalart.astrum.repository.content.utils.SearchComponentPathFilter;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import static org.globalart.astrum.content.maven.internal.search.table.MavenSearchComponentPathFilter.MavenType.getMavenTypes;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier(Maven2Format.NAME)
public class MavenSearchComponentPathFilter
    implements SearchComponentPathFilter
{
  enum MavenType
  {
    POM(".pom"),
    WAR(".war"),
    JAR(".jar"),
    EAR(".ear"),
    AAR(".aar"),
    ZIP(".zip"),
    TARGZ(".tar.gz");

    private final String mavenType;

    MavenType(final String mavenType) {
      this.mavenType = mavenType;
    }

    private String getMavenType() {
      return mavenType;
    }

    static List<String> getMavenTypes() {
      return Arrays.stream(MavenType.values())
          .map(MavenType::getMavenType)
          .collect(Collectors.toList());
    }
  }

  @Override
  public boolean shouldFilterPathExtension(final String path) {
    return getMavenTypes().stream().noneMatch(path::endsWith);
  }
}
