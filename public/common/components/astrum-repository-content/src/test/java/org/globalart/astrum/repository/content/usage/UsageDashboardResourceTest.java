package org.globalart.astrum.repository.content.usage;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.content.usage.UsageDashboardXO.DayXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.RepositoryXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.TotalsXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.UnusedXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.UsageComponentXO;
import org.globalart.astrum.rest.WebApplicationMessageException;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.hasKey;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class UsageDashboardResourceTest
{
  private static final UsageDashboardXO DASHBOARD = new UsageDashboardXO(7, "2026-10-05", "2026-10-11", 180, 90, null,
      List.of(new RepositoryXO("raw-hosted", "raw", "hosted")),
      new TotalsXO(1, 1, 0, 0, List.of(), 2, 3, 4, 0, 5, 1, 2, 3, 2, 1),
      List.of(new DayXO("2026-10-11", 1)),
      List.of(new UsageComponentXO("abc", "raw-hosted", "raw", null, "file.txt", null, 1, "2026-10-11T07:00:00Z",
          "admin", "file.txt")),
      List.of(),
      List.of(),
      List.of(),
      List.of(),
      new UnusedXO(0, 0, 0, 0, List.of()));

  @Mock
  private UsageDashboardService service;

  private UsageDashboardResource underTest;

  @Before
  public void setUp() {
    underTest = new UsageDashboardResource(service);
  }

  @Test
  public void returnsTheFiguresOfAllRepositories() {
    when(service.dashboard(7, null)).thenReturn(Optional.of(DASHBOARD));

    assertThat(underTest.getUsage(7, null), sameInstance(DASHBOARD));
    assertThat(underTest.getUsage(7, " "), sameInstance(DASHBOARD));
  }

  @Test
  public void returnsTheFiguresOfOneRepository() {
    when(service.dashboard(30, "raw-hosted")).thenReturn(Optional.of(DASHBOARD));

    assertThat(underTest.getUsage(30, "raw-hosted"), sameInstance(DASHBOARD));
    verify(service).dashboard(30, "raw-hosted");
  }

  @Test
  public void answersNotFoundForARepositoryTheUserMayNotBrowse() {
    when(service.dashboard(30, "secret")).thenReturn(Optional.empty());

    WebApplicationMessageException thrown =
        assertThrows(WebApplicationMessageException.class, () -> underTest.getUsage(30, "secret"));
    assertThat(thrown.getResponse().getStatus(), is(404));
  }

  @Test
  public void writesTheFiguresAsJson() throws Exception {
    ObjectMapper mapper = new ObjectMapper();

    Map<String, Object> json = mapper.readValue(mapper.writeValueAsString(DASHBOARD), new TypeReference<>() { });

    assertThat(json, hasEntry("days", 7));
    assertThat(json, hasEntry("from", "2026-10-05"));
    assertThat(json, hasKey("downloadsPerDay"));
    assertThat(((Map<?, ?>) json.get("totals")).get("downloadsLast7Days"), is(2));
    Map<?, ?> component = (Map<?, ?>) ((List<?>) json.get("topComponents")).get(0);
    assertThat(component.get("browsePath"), is("file.txt"));
    assertThat(component.get("repository"), is("raw-hosted"));
  }
}
