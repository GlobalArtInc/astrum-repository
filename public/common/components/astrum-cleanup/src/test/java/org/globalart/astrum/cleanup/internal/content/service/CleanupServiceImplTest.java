
package org.globalart.astrum.cleanup.internal.content.service;

import java.util.Map;
import java.util.function.BooleanSupplier;
import java.util.stream.Stream;

import org.globalart.astrum.cleanup.content.search.CleanupBrowseServiceFactory;
import org.globalart.astrum.cleanup.content.search.CleanupComponentBrowse;
import org.globalart.astrum.cleanup.internal.method.CleanupMethod;
import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.cleanup.storage.CleanupPolicyStorage;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cleanup.CleanupFeatureCheck;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.task.DeletionProgress;
import org.globalart.astrum.repository.types.GroupType;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

import static com.google.common.collect.Sets.newLinkedHashSet;
import static java.util.Arrays.asList;
import static java.util.Collections.emptyMap;
import static java.util.Collections.singletonMap;
import static java.util.stream.Stream.empty;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.mockito.hamcrest.MockitoHamcrest.argThat;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.testcommon.matchers.AstrumMatchers.streamContains;

public class CleanupServiceImplTest
{
  @Rule
  public MockitoRule mockitoRule = MockitoJUnit.rule().silent();

  private static final String POLICY_1_NAME = "policy1";

  private static final String POLICY_2_NAME = "policy2";

  private static final int RETRY_LIMIT = 3;

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private Repository repository1, repository2, repository3;

  @Mock
  private CleanupComponentBrowse browseService;

  @Mock
  private CleanupPolicyStorage cleanupPolicyStorage;

  @Mock
  private CleanupPolicy cleanupPolicy1, cleanupPolicy2;

  @Mock
  private CleanupMethod cleanupMethod;

  @Mock
  private Format format;

  @Mock
  private FluentComponent component1, component2, component3;

  @Mock
  private Type type;

  @Mock
  private BooleanSupplier cancelledCheck;

  @Mock
  private DeletionProgress deletionProgress;

  @Mock
  private CleanupBrowseServiceFactory cleanupBrowseFactory;

  @Mock
  private CleanupFeatureCheck cleanupFeatureCheck;

  private CleanupServiceImpl underTest;

  @Before
  public void setup() throws Exception {
    when(cleanupBrowseFactory.get(any())).thenReturn(browseService);

    underTest = new CleanupServiceImpl(repositoryManager, cleanupPolicyStorage, cleanupMethod,
        new GroupType(), RETRY_LIMIT, cleanupBrowseFactory, cleanupFeatureCheck);

    setupRepository(repository1, POLICY_1_NAME);
    setupRepository(repository2, POLICY_2_NAME);
    setupRepository(repository3, null);

    when(repositoryManager.browse()).thenReturn(ImmutableList.of(repository1, repository2));

    when(cleanupPolicyStorage.get(POLICY_1_NAME)).thenReturn(cleanupPolicy1);
    when(cleanupPolicyStorage.get(POLICY_2_NAME)).thenReturn(cleanupPolicy2);

    when(cleanupPolicy1.getCriteria()).thenReturn(ImmutableMap.of(LAST_BLOB_UPDATED_KEY, "1"));
    when(cleanupPolicy2.getCriteria()).thenReturn(ImmutableMap.of(LAST_DOWNLOADED_KEY, "2"));

    when(browseService.browse(cleanupPolicy1, repository1))
        .thenReturn(ImmutableList.of(component1, component2).stream());
    when(browseService.browse(cleanupPolicy2, repository2)).thenReturn(ImmutableList.of(component3).stream());

    when(cancelledCheck.getAsBoolean()).thenReturn(false);

    when(deletionProgress.isFailed()).thenReturn(false);
    when(cleanupMethod.run(any(), any(), any())).thenReturn(deletionProgress);

    when(repository1.getFormat()).thenReturn(format);
    when(repository2.getFormat()).thenReturn(format);
    when(repository3.getFormat()).thenReturn(format);
    when(format.getValue()).thenReturn("maven2");

    when(cleanupFeatureCheck.isRetainSupported(any())).thenReturn(true);
  }

  @Test
  public void fetchPolicyForEachRepositoryAndRunCleanup() throws Exception {
    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod).run(eq(repository1), argThat(streamContains(component1, component2)), eq(cancelledCheck));
    verify(cleanupMethod).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void fetchMultiplePoliciesForEachRepositoryAndRunCleanup() {
    String[] policyNamesForRepo1 = {"abc", "def", "ghi"};
    Stream<FluentComponent> componentsForRepo1 = setupForMultiplePolicies(repository1, policyNamesForRepo1);

    String[] policyNamesForRepo2 = {"qwe", "rty", "uio"};
    Stream<FluentComponent> componentsForRepo2 = setupForMultiplePolicies(repository2, policyNamesForRepo2);

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, times(policyNamesForRepo1.length)).run(repository1, componentsForRepo1, cancelledCheck);
    verify(cleanupMethod, times(policyNamesForRepo2.length)).run(repository2, componentsForRepo2, cancelledCheck);
  }

  @Test
  public void ignoreRepositoryWhenPolicyNull() throws Exception {
    when(cleanupPolicyStorage.get(POLICY_2_NAME)).thenReturn(null);

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod).run(eq(repository1), argThat(streamContains(component1, component2)), eq(cancelledCheck));
  }

  @Test
  public void ignoreRepositoryWhenPolicyNameNull() throws Exception {
    setupRepository(repository2, new String[]{null});
    when(cleanupPolicyStorage.get(null)).thenThrow(new NullPointerException());

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod).run(eq(repository1), argThat(streamContains(component1, component2)), eq(cancelledCheck));
  }

  @Test
  public void skipPolicyWithExclusionIfUnsupportedFormat() {
    // Covers CleanupServiceImpl.shouldSkip(): policies with exclusion (retain)
    // criteria must be skipped when the format does not support retain.
    when(cleanupFeatureCheck.isRetainSupported(any())).thenReturn(false);
    when(repositoryManager.browse()).thenReturn(ImmutableList.of(repository1));
    when(cleanupPolicy1.getCriteria()).thenReturn(
        ImmutableMap.of(LAST_BLOB_UPDATED_KEY, "1", "retain", "3", "sortBy", "version"));

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, never())
        .run(eq(repository1), argThat(streamContains(component1, component2)), eq(cancelledCheck));
  }

  @Test
  public void ignoreRepositoryWhenPolicyNameListIsNull() throws Exception {
    when(repositoryManager.browse()).thenReturn(ImmutableList.of(repository3));

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, never()).run(eq(repository3), argThat(streamContains(component2)), eq(cancelledCheck));
  }

  @Test
  public void ignoreRepositoryWhenPolicyNameAttributeIsNotPresent() throws Exception {
    repository1.getConfiguration().setAttributes(emptyMap());

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void ignoreRepositoryWhenAttributesNull() throws Exception {
    when(repository1.getConfiguration()).thenReturn(mock(Configuration.class));

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void ignoreRepositoryWhenCancelled() throws Exception {
    doAnswer(i -> {
      when(cancelledCheck.getAsBoolean()).thenReturn(true);
      return deletionProgress;
    }).when(cleanupMethod).run(eq(repository1), any(), eq(cancelledCheck));

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod).run(eq(repository1), argThat(streamContains(component1, component2)), eq(cancelledCheck));
    verify(cleanupMethod, never()).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void doNothingWhenNoComponentsFound() throws Exception {
    when(browseService.browse(cleanupPolicy1, repository1)).thenReturn(empty());
    when(browseService.browse(cleanupPolicy2, repository2)).thenReturn(empty());

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, never()).run(eq(repository1), argThat(streamContains(component1, component2)),
        eq(cancelledCheck));
    verify(cleanupMethod, never()).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void doNotDeleteAnythingWhenCriteriaEmpty() throws Exception {
    when(cleanupPolicy1.getCriteria()).thenReturn(emptyMap());
    when(cleanupPolicy2.getCriteria()).thenReturn(emptyMap());

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, never()).run(eq(repository1), argThat(streamContains(component1, component2)),
        eq(cancelledCheck));
    verify(cleanupMethod, never()).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void retryDeletionIfFailure() {
    when(deletionProgress.isFailed()).thenReturn(true).thenReturn(false);
    when(repositoryManager.browse()).thenReturn(ImmutableList.of(repository1));
    when(cleanupMethod.run(any(), any(), any())).thenReturn(deletionProgress);

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, times(2)).run(any(), any(), any());
  }

  @Test
  public void retryAttemptsExceeded() {
    when(deletionProgress.isFailed()).thenReturn(true);
    when(repositoryManager.browse()).thenReturn(ImmutableList.of(repository1));
    when(cleanupMethod.run(any(), any(), any())).thenReturn(deletionProgress);

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, times(3)).run(any(), any(), any());
  }

  @Test
  public void cleanupRetriedOnScrollTimeout() {
    when(cleanupMethod.run(any(), any(), any()))
        .thenThrow(new RuntimeException())
        .thenReturn(deletionProgress);

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, times(2)).run(eq(repository1), argThat(streamContains(component1, component2)),
        eq(cancelledCheck));
    verify(cleanupMethod).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void cleanupFailed() {
    when(cleanupMethod.run(any(), any(), any())).thenThrow(new RuntimeException());

    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, times(3)).run(eq(repository1), argThat(streamContains(component1, component2)),
        eq(cancelledCheck));
    verify(cleanupMethod, times(3)).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  @Test
  public void cleanupFailedSearchContextMissingException() {
    when(cleanupMethod.run(any(), any(), any()))
        .thenThrow(new RuntimeException());
    underTest.cleanup(cancelledCheck);

    verify(cleanupMethod, times(3)).run(eq(repository1), argThat(streamContains(component1, component2)),
        eq(cancelledCheck));
    verify(cleanupMethod, times(3)).run(eq(repository2), argThat(streamContains(component3)), eq(cancelledCheck));
  }

  private void setupRepository(final Repository repository, final String... policyName) {
    Configuration repositoryConfig = mock(Configuration.class);
    when(repository.getConfiguration()).thenReturn(repositoryConfig);

    ImmutableMap<String, Map<String, Object>> attributes = ImmutableMap
        .of("cleanup", singletonMap("policyName", policyName != null ? newLinkedHashSet(asList(policyName)) : null));
    when(repositoryConfig.getAttributes()).thenReturn(attributes);

    when(repository.getType()).thenReturn(type);

  }

  private Stream<FluentComponent> setupForMultiplePolicies(final Repository repository, final String... policyNames) {
    setupRepository(repository, policyNames);
    return setupComponents(repository, policyNames);
  }

  private Stream<FluentComponent> setupComponents(
      final Repository repository,
      final String... policyNames)
  {
    Stream<FluentComponent> components =
        ImmutableList.of(mock(FluentComponent.class), mock(FluentComponent.class)).stream();

    asList(policyNames).forEach(policyName -> {
      CleanupPolicy cleanupPolicy = mock(CleanupPolicy.class);
      when(cleanupPolicy.getCriteria()).thenReturn(ImmutableMap.of(LAST_BLOB_UPDATED_KEY, "1"));
      when(cleanupPolicyStorage.get(policyName)).thenReturn(cleanupPolicy);
      when(browseService.browse(cleanupPolicy, repository)).thenReturn(components);
    });

    return components;
  }
}
