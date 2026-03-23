
package org.globalart.astrum.content.maven.internal.search.table;

import java.util.Collections;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Stream;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import static org.globalart.astrum.repository.maven.internal.Attributes.P_ARTIFACT_ID;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_BASE_VERSION;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_CLASSIFIER;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_EXTENSION;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_GROUP_ID;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier(Maven2Format.NAME)
public class MavenSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchTableData, final Asset asset) {

    Object formatAttributes = asset.attributes().get(Maven2Format.NAME);

    @SuppressWarnings("unchecked")
    Map<String, String> attributes =
        formatAttributes instanceof Map ? (Map<String, String>) formatAttributes : Collections.emptyMap();

    Optional.ofNullable(attributes.get(P_BASE_VERSION))
        .ifPresent(token -> searchTableData.addFormatFieldValue1(token, true));
    Optional.ofNullable(attributes.get(P_EXTENSION))
        .ifPresent(searchTableData::addFormatFieldValue2);
    Optional.ofNullable(attributes.get(P_CLASSIFIER))
        .ifPresent(searchTableData::addFormatFieldValue3);

    buildGavec(searchTableData, attributes);
  }

  private static void buildGavec(final SearchRecord searchTableData, final Map<String, String> attributes) {
    Stream.of(attributes.get(P_GROUP_ID),
        attributes.get(P_ARTIFACT_ID),
        attributes.get(P_BASE_VERSION),
        attributes.get(P_EXTENSION),
        attributes.get(P_CLASSIFIER))
        .filter(Objects::nonNull)
        .forEach(field -> searchTableData.addFormatFieldValue4(field, true));
  }
}
