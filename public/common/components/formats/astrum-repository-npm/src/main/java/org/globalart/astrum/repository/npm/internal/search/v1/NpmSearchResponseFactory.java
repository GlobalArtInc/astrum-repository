
package org.globalart.astrum.repository.npm.internal.search.v1;

import java.util.Arrays;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.joda.time.DateTime;
import org.joda.time.DateTimeZone;
import org.joda.time.format.DateTimeFormat;
import org.joda.time.format.DateTimeFormatter;
import org.springframework.stereotype.Component;

import static java.util.Collections.emptyList;
import static java.util.Collections.singletonList;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_AUTHOR;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_BUGS_URL;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_DESCRIPTION;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_HOMEPAGE;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_KEYWORDS;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_NAME;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_REPOSITORY_URL;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_VERSION;

@Component
public class NpmSearchResponseFactory
{
  private static final DateTimeFormatter SEARCH_RESPONSE_PACKAGE_DATE_FORMAT =
      DateTimeFormat.forPattern("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'");

  private static final DateTimeFormatter SEARCH_RESPONSE_DATE_FORMAT =
      DateTimeFormat.forPattern("EEE MMM dd yyyy HH:mm:ss 'GMT'Z '('z')'");

  private static final Double DEFAULT_SCORE = 0.0;

  public NpmSearchResponse buildEmptyResponse() {
    NpmSearchResponse response = new NpmSearchResponse();
    response.setObjects(emptyList());
    response.setTime(formatSearchResponseDate(DateTime.now()));
    response.setTotal(0);
    return response;
  }

  public NpmSearchResponse buildResponseForObjects(final List<NpmSearchResponseObject> objects) {
    NpmSearchResponse response = new NpmSearchResponse();
    response.setObjects(objects);
    response.setTime(formatSearchResponseDate(DateTime.now()));
    response.setTotal(objects.size());
    return response;
  }

  public NpmSearchResponseObject buildSearchResponseObject(
      final Map<String, Object> npmAttributes,
      @Nullable final DateTime lastModified,
      final double searchScore)
  {
    NpmSearchResponsePerson person = buildPerson(string(npmAttributes, P_AUTHOR));

    NpmSearchResponsePackageLinks links = new NpmSearchResponsePackageLinks();
    links.setBugs(string(npmAttributes, P_BUGS_URL));
    links.setHomepage(string(npmAttributes, P_HOMEPAGE));
    links.setRepository(string(npmAttributes, P_REPOSITORY_URL));

    NpmSearchResponsePackage searchPackage = new NpmSearchResponsePackage();
    searchPackage.setDate(lastModified == null ? null : formatSearchResponsePackageDate(lastModified));
    searchPackage.setName(string(npmAttributes, P_NAME));
    searchPackage.setVersion(string(npmAttributes, P_VERSION));
    searchPackage.setDescription(string(npmAttributes, P_DESCRIPTION));
    searchPackage.setKeywords(keywords(npmAttributes.get(P_KEYWORDS)));
    searchPackage.setPublisher(person);
    searchPackage.setMaintainers(person == null ? emptyList() : singletonList(person));
    searchPackage.setLinks(links);

    NpmSearchResponseObject searchObject = new NpmSearchResponseObject();
    searchObject.setPackageEntry(searchPackage);
    searchObject.setSearchScore(searchScore);
    searchObject.setScore(buildPackageScore());
    return searchObject;
  }

  public static List<String> keywords(@Nullable final Object value) {
    if (value instanceof Collection<?> collection) {
      return collection.stream().map(String::valueOf).collect(Collectors.toList());
    }
    if (value == null) {
      return emptyList();
    }
    String keywords = value.toString().trim();
    if (keywords.isEmpty()) {
      return emptyList();
    }
    return Arrays.asList(keywords.split("\\s+"));
  }

  @Nullable
  private static String string(final Map<String, Object> attributes, final String key) {
    Object value = attributes.get(key);
    return value == null ? null : value.toString();
  }

  private NpmSearchResponseScore buildPackageScore() {
    NpmSearchResponseScoreDetail scoreDetail = new NpmSearchResponseScoreDetail();
    scoreDetail.setMaintenance(DEFAULT_SCORE);
    scoreDetail.setPopularity(DEFAULT_SCORE);
    scoreDetail.setQuality(DEFAULT_SCORE);

    NpmSearchResponseScore score = new NpmSearchResponseScore();
    score.setFinalScore(DEFAULT_SCORE);
    score.setDetail(scoreDetail);
    return score;
  }

  @Nullable
  private NpmSearchResponsePerson buildPerson(@Nullable final String author) {
    String username = parseAuthorName(author);
    String email = parseAuthorEmail(author);
    if (username == null && email == null) {
      return null;
    }
    NpmSearchResponsePerson person = new NpmSearchResponsePerson();
    person.setUsername(username);
    person.setEmail(email);
    return person;
  }

  @Nullable
  private static String parseAuthorName(@Nullable final String author) {
    if (author == null) {
      return null;
    }
    int endIndex = author.indexOf('<');
    if (endIndex == -1) {
      endIndex = author.indexOf('(');
    }
    String username = endIndex == -1 ? author.trim() : author.substring(0, endIndex).trim();
    return username.isEmpty() ? null : username;
  }

  @Nullable
  private static String parseAuthorEmail(@Nullable final String author) {
    if (author == null) {
      return null;
    }
    int startIndex = author.indexOf('<');
    if (startIndex == -1) {
      return null;
    }
    int endIndex = author.indexOf('>', startIndex);
    if (endIndex == -1) {
      return null;
    }
    String email = author.substring(startIndex + 1, endIndex).trim();
    return email.isEmpty() ? null : email;
  }

  private String formatSearchResponsePackageDate(final DateTime dateTime) {
    return SEARCH_RESPONSE_PACKAGE_DATE_FORMAT.print(dateTime.toDateTime(DateTimeZone.UTC));
  }

  private String formatSearchResponseDate(final DateTime dateTime) {
    return SEARCH_RESPONSE_DATE_FORMAT.print(dateTime.toDateTime(DateTimeZone.UTC));
  }
}
