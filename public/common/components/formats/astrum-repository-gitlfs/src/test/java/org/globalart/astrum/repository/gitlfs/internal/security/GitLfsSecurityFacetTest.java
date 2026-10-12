package org.globalart.astrum.repository.gitlfs.internal.security;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;
import org.globalart.astrum.selector.VariableSource;

import org.apache.shiro.authz.AuthorizationException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class GitLfsSecurityFacetTest
{
  private static final String OID = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";

  @Mock
  private GitLfsFormatSecurityContributor securityContributor;

  @Mock
  private VariableResolverAdapter variableResolverAdapter;

  @Mock
  private ContentPermissionChecker contentPermissionChecker;

  @Mock
  private Repository repository;

  @Mock
  private VariableSource variableSource;

  private GitLfsSecurityFacet underTest;

  @BeforeEach
  void setUp() throws Exception {
    when(repository.getName()).thenReturn("lfs");
    when(repository.getFormat()).thenReturn(new GitLfsFormat());
    when(variableResolverAdapter.fromRequest(any(), eq(repository))).thenReturn(variableSource);
    underTest = new GitLfsSecurityFacet(securityContributor, variableResolverAdapter, contentPermissionChecker);
    underTest.attach(repository);
  }

  @Test
  void transfersInBatchesNeedOnlyReadUpFront() {
    assertThat(action("POST", "/info/lfs/objects/batch"), is(BreadActions.READ));
    assertThat(action("POST", "/objects/batch"), is(BreadActions.READ));
    assertThat(action("POST", "/info/lfs/objects/verify"), is(BreadActions.READ));
  }

  @Test
  void objectsAreReadAndAdded() {
    assertThat(action("GET", "/info/lfs/objects/" + OID), is(BreadActions.READ));
    assertThat(action("HEAD", "/objects/4d/7a/" + OID), is(BreadActions.READ));
    assertThat(action("PUT", "/info/lfs/objects/" + OID), is(BreadActions.ADD));
  }

  @Test
  void locksNeedAddExceptToList() {
    assertThat(action("GET", "/info/lfs/locks"), is(BreadActions.READ));
    assertThat(action("POST", "/info/lfs/locks"), is(BreadActions.ADD));
    assertThat(action("POST", "/info/lfs/locks/verify"), is(BreadActions.ADD));
    assertThat(action("POST", "/info/lfs/locks/3/unlock"), is(BreadActions.ADD));
  }

  @Test
  void otherPathsKeepTheMethodMapping() {
    assertThat(action("GET", "/"), is(BreadActions.READ));
    assertThat(action("DELETE", "/whatever"), is(BreadActions.DELETE));
  }

  @Test
  void checksObjectTransfersAgainstTheAssetPath() {
    when(contentPermissionChecker.isPermitted(eq("lfs"), eq("gitlfs"), eq(BreadActions.ADD), eq(variableSource),
        any())).thenReturn(true);

    underTest.ensurePermitted(request("PUT", "/info/lfs/objects/" + OID));

    ArgumentCaptor<Request> checked = ArgumentCaptor.forClass(Request.class);
    verify(variableResolverAdapter).fromRequest(checked.capture(), eq(repository));
    assertThat(checked.getValue().getPath(), is("/objects/4d/7a/" + OID));
  }

  @Test
  void refusesWhenThePrivilegeIsMissing() {
    assertThrows(AuthorizationException.class,
        () -> underTest.ensurePermitted(request("POST", "/info/lfs/locks")));
  }

  @Test
  void checksSingleObjectsForTheBatchHandler() {
    when(contentPermissionChecker.isPermitted("lfs", "gitlfs", BreadActions.ADD, variableSource)).thenReturn(true);

    assertThat(underTest.isPermitted(request("POST", "/info/lfs/objects/batch"), OID, BreadActions.ADD), is(true));
    assertThat(underTest.isPermitted(request("POST", "/info/lfs/objects/batch"), OID, BreadActions.READ), is(false));

    ArgumentCaptor<Request> checked = ArgumentCaptor.forClass(Request.class);
    verify(variableResolverAdapter, times(2)).fromRequest(checked.capture(), eq(repository));
    assertThat(checked.getValue().getPath(), is("/objects/4d/7a/" + OID));
  }

  private String action(final String method, final String path) {
    return underTest.action(request(method, path));
  }

  private static Request request(final String method, final String path) {
    return new Request.Builder().action(method).path(path).build();
  }
}
