
package org.globalart.astrum.cleanup.internal.datastore.search.criteria;

import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.function.BiPredicate;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.utils.PreReleaseEvaluator;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.MockedStatic;
import org.mockito.Mockito;

import static java.util.Collections.emptyList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class PrereleaseCleanupEvaluatorTest

{
  @Mock
  private Repository repository;

  @Mock
  private PreReleaseEvaluator preReleaseEvaluator;

  private PrereleaseCleanupEvaluator underTest;

  private MockedStatic<QualifierUtil> mockedStatic;

  @BeforeEach
  void setup() {
    mockedStatic = Mockito.mockStatic(QualifierUtil.class);
    List<PreReleaseEvaluator> preReleaseEvaluators = List.of(preReleaseEvaluator);
    Map<String, PreReleaseEvaluator> matchers = Collections.singletonMap("test-format", preReleaseEvaluator);
    when(QualifierUtil.buildQualifierBeanMap(preReleaseEvaluators)).thenReturn(matchers);
    underTest = new PrereleaseCleanupEvaluator(preReleaseEvaluators);
    setupRepositoryAndEvaluator();
  }

  @AfterEach
  void tearDown() {
    mockedStatic.close();
  }

  private void setupRepositoryAndEvaluator() {
    Format formatMock = mock(Format.class);
    when(repository.getFormat()).thenReturn(formatMock);
    when(formatMock.getValue()).thenReturn("test-format");
  }

  @ParameterizedTest
  @CsvSource({
      "true, true, true",
      "true, false, false",
      "false, false, true"
  })
  void testPredicateBehavior(final String value, final boolean isPreRelease, final boolean expectedResult) {
    when(preReleaseEvaluator.isPreRelease(any(Component.class), any(Iterable.class))).thenReturn(isPreRelease);
    BiPredicate<Component, Iterable<Asset>> predicate = underTest.getPredicate(repository, value);
    assertThat(predicate.test(mock(Component.class), emptyList()), is(expectedResult));
  }
}
