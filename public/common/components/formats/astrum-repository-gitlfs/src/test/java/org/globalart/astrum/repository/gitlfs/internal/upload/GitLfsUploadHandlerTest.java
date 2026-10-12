package org.globalart.astrum.repository.gitlfs.internal.upload;

import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsContentFacet;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.AssetUpload;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.security.BreadActions;
import org.globalart.astrum.selector.VariableSource;

import com.google.common.hash.HashCode;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class GitLfsUploadHandlerTest
{
  private static final String OID = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";

  @Mock
  private ContentPermissionChecker contentPermissionChecker;

  @Mock
  private VariableResolverAdapter variableResolverAdapter;

  @Mock
  private Repository repository;

  @Mock
  private GitLfsContentFacet content;

  @Mock
  private TempBlob blob;

  @Mock
  private FluentAsset asset;

  @Mock
  private PartPayload payload;

  private GitLfsUploadHandler underTest;

  @BeforeEach
  void setUp() {
    underTest = new GitLfsUploadHandler(contentPermissionChecker, variableResolverAdapter, Set.of());
    when(repository.getName()).thenReturn("lfs");
    when(repository.facet(GitLfsContentFacet.class)).thenReturn(content);
    when(content.ingest(payload)).thenReturn(blob);
    when(blob.getHashes()).thenReturn(Map.of(SHA256, HashCode.fromString(OID)));
    when(content.put(blob)).thenReturn(asset);
    when(asset.download()).thenReturn(new Content(new BytesPayload("x".getBytes(UTF_8), "text/plain")));
    when(variableResolverAdapter.fromCoordinates(eq("gitlfs"), any(), any())).thenReturn(new VariableSource(List.of()));
  }

  @Test
  void definesASingleFileUploadWithoutFields() {
    assertThat(underTest.getDefinition().getFormat(), is("gitlfs"));
    assertThat(underTest.getDefinition().isMultipleUpload(), is(false));
    assertThat(underTest.getDefinition().getComponentFields(), is(empty()));
    assertThat(underTest.getDefinition().getAssetFields(), is(empty()));
  }

  @Test
  void storesTheFileAsTheObjectItHashesTo() throws Exception {
    when(contentPermissionChecker.isPermitted(eq("lfs"), eq("gitlfs"), eq(BreadActions.EDIT), any()))
        .thenReturn(true);

    UploadResponse response = underTest.handle(repository, upload());

    assertThat(response.getAssetPaths(), contains("/objects/4d/7a/" + OID));
    verify(content).put(blob);
    verify(blob).close();
  }

  @Test
  void refusesUsersWithoutThePrivilege() {
    assertThrows(ValidationErrorsException.class, () -> underTest.handle(repository, upload()));

    verify(content, never()).put(any(TempBlob.class));
  }

  private ComponentUpload upload() {
    AssetUpload assetUpload = new AssetUpload();
    assetUpload.setPayload(payload);
    ComponentUpload upload = new ComponentUpload();
    upload.setAssetUploads(List.of(assetUpload));
    return upload;
  }
}
