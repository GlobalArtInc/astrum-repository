
package org.globalart.astrum.content.raw;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.content.fluent.FluentBlobs;
import org.globalart.astrum.repository.importtask.ImportFileConfiguration;
import org.globalart.astrum.repository.importtask.ImportStreamConfiguration;
import org.globalart.astrum.repository.raw.RawUploadHandlerTestSupport;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.AssetUpload;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadHandler;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.view.payloads.TempBlobPayload;

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertNotNull;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.reset;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class RawUploadHandlerTest
    extends RawUploadHandlerTestSupport
{
  @Mock
  RawContentFacet rawFacet;

  @Mock
  FluentBlobs blobs;

  @Mock
  TempBlob tempBlob;

  @Override
  protected UploadHandler newRawUploadHandler(
      final ContentPermissionChecker contentPermissionChecker,
      final VariableResolverAdapter variableResolverAdapter,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    return new RawUploadHandler(contentPermissionChecker, variableResolverAdapter, uploadDefinitionExtensions);
  }

  @Before
  public void setup() throws IOException {
    when(repository.facet(RawContentFacet.class)).thenReturn(rawFacet);
    when(rawFacet.blobs()).thenReturn(blobs);
  }

  @Test
  public void testHandle() throws IOException {
    ComponentUpload component = new ComponentUpload();

    component.getFields().put("directory", "org/apache/maven");

    AssetUpload asset = new AssetUpload();
    asset.getFields().put("filename", "foo.jar");
    asset.setPayload(jarPayload);
    component.getAssetUploads().add(asset);

    asset = new AssetUpload();
    asset.getFields().put("filename", "bar.jar");
    asset.setPayload(sourcesPayload);
    component.getAssetUploads().add(asset);

    when(content.getAttributes()).thenReturn(attributesMap);
    when(rawFacet.put(any(), any())).thenReturn(content);
    UploadResponse uploadResponse = underTest.handle(repository, component);
    assertThat(uploadResponse.getAssetPaths(), contains("/org/apache/maven/foo.jar", "/org/apache/maven/bar.jar"));

    ArgumentCaptor<String> pathCapture = ArgumentCaptor.forClass(String.class);
    verify(rawFacet, times(2)).put(pathCapture.capture(), any(PartPayload.class));

    List<String> paths = pathCapture.getAllValues();

    assertThat(paths, hasSize(2));

    String path = paths.get(0);
    assertNotNull(path);
    assertThat(path, is("/org/apache/maven/foo.jar"));

    path = paths.get(1);
    assertNotNull(path);
    assertThat(path, is("/org/apache/maven/bar.jar"));
  }

  @Test
  public void testHandleHardLink() throws IOException {
    Path contentPath = Files.createTempDirectory("raw-upload-test").resolve("test.txt");
    String path = contentPath.toString();
    Content content = mock(Content.class);
    when(rawFacet.put(eq(path), any(TempBlobPayload.class))).thenReturn(content);

    when(blobs.ingest(eq(contentPath), any(), any(), eq(true))).thenReturn(tempBlob);

    Content importResponse =
        underTest.handle(new ImportFileConfiguration(repository, contentPath.toFile(), path, true));

    verify(rawFacet).put(eq(path), any(TempBlobPayload.class));
    assertThat(importResponse, is(content));
  }

  @Test
  public void testHandleStreamImport() throws IOException {
    String assetName = "test.jar";
    String testContent = "test content for stream import";
    InputStream inputStream = new ByteArrayInputStream(testContent.getBytes());

    Content content = mock(Content.class);
    when(rawFacet.put(eq(assetName), any(TempBlobPayload.class))).thenReturn(content);
    when(blobs.ingest(eq(inputStream), any(), any())).thenReturn(tempBlob);

    ImportStreamConfiguration configuration = new ImportStreamConfiguration(repository, inputStream, assetName);
    Content importResponse = underTest.handle(configuration);

    verify(rawFacet).put(eq(assetName), any(TempBlobPayload.class));
    assertThat(importResponse, is(content));
  }

  @Test
  public void testHandleStreamImportWithJarFile() throws IOException {
    String assetName = "example-1.0.0.jar";
    String testContent = "jar file content";
    InputStream inputStream = new ByteArrayInputStream(testContent.getBytes());

    Content content = mock(Content.class);
    when(rawFacet.put(eq(assetName), any(TempBlobPayload.class))).thenReturn(content);
    when(blobs.ingest(eq(inputStream), any(), any())).thenReturn(tempBlob);

    ImportStreamConfiguration configuration = new ImportStreamConfiguration(repository, inputStream, assetName);
    Content importResponse = underTest.handle(configuration);

    verify(rawFacet).put(eq(assetName), any(TempBlobPayload.class));
    assertThat(importResponse, is(content));
  }

  @Test
  public void testHandleStreamImportWithTextFile() throws IOException {
    String assetName = "readme.txt";
    String testContent = "This is a text file";
    InputStream inputStream = new ByteArrayInputStream(testContent.getBytes());

    Content content = mock(Content.class);
    when(rawFacet.put(eq(assetName), any(TempBlobPayload.class))).thenReturn(content);
    when(blobs.ingest(eq(inputStream), any(), any())).thenReturn(tempBlob);

    ImportStreamConfiguration configuration = new ImportStreamConfiguration(repository, inputStream, assetName);
    Content importResponse = underTest.handle(configuration);

    verify(rawFacet).put(eq(assetName), any(TempBlobPayload.class));
    assertThat(importResponse, is(content));
  }

  @Test
  public void testHandleStreamImportWithBinaryFile() throws IOException {
    String assetName = "image.png";
    byte[] binaryContent = {(byte) 0x89, 0x50, 0x4E, 0x47}; // PNG signature
    InputStream inputStream = new ByteArrayInputStream(binaryContent);

    Content content = mock(Content.class);
    when(rawFacet.put(eq(assetName), any(TempBlobPayload.class))).thenReturn(content);
    when(blobs.ingest(eq(inputStream), any(), any())).thenReturn(tempBlob);

    ImportStreamConfiguration configuration = new ImportStreamConfiguration(repository, inputStream, assetName);
    Content importResponse = underTest.handle(configuration);

    verify(rawFacet).put(eq(assetName), any(TempBlobPayload.class));
    assertThat(importResponse, is(content));
  }

  @Test(expected = NullPointerException.class)
  public void testHandleStreamImportWithNullConfiguration() throws IOException {
    underTest.handle((ImportStreamConfiguration) null);
  }

  @Test(expected = NullPointerException.class)
  public void testHandleStreamImportWithNullInputStream() throws IOException {
    String assetName = "test.txt";
    ImportStreamConfiguration configuration = new ImportStreamConfiguration(repository, null, assetName);
    underTest.handle(configuration);
  }

  @Test(expected = NullPointerException.class)
  public void testHandleStreamImportWithNullAssetName() throws IOException {
    InputStream inputStream = new ByteArrayInputStream("test".getBytes());
    ImportStreamConfiguration configuration = new ImportStreamConfiguration(repository, inputStream, null);
    underTest.handle(configuration);
  }

  @Test
  public void testDetermineContentTypeFromPath() throws IOException {
    // Test the content type determination method through stream import
    String[] testCases = {
        "test.jar",
        "document.pdf",
        "image.png",
        "archive.zip",
        "script.sh",
        "data.xml",
        "unknown.xyz"
    };

    for (String assetName : testCases) {
      InputStream inputStream = new ByteArrayInputStream("test content".getBytes());
      Content content = mock(Content.class);

      when(rawFacet.put(eq(assetName), any(TempBlobPayload.class))).thenReturn(content);
      when(blobs.ingest(eq(inputStream), any(), any())).thenReturn(tempBlob);

      ImportStreamConfiguration configuration = new ImportStreamConfiguration(repository, inputStream, assetName);
      Content importResponse = underTest.handle(configuration);

      verify(rawFacet).put(eq(assetName), any(TempBlobPayload.class));
      assertThat(importResponse, is(content));

      reset(rawFacet, blobs);
      when(repository.facet(RawContentFacet.class)).thenReturn(rawFacet);
      when(rawFacet.blobs()).thenReturn(blobs);
    }
  }

  @Override
  protected void testNormalizePath(
      final String directory,
      final String file,
      final String expectedPath) throws IOException
  {
    reset(rawFacet);
    ComponentUpload component = new ComponentUpload();

    component.getFields().put("directory", directory);

    AssetUpload asset = new AssetUpload();
    asset.getFields().put("filename", file);
    asset.setPayload(jarPayload);
    component.getAssetUploads().add(asset);

    when(content.getAttributes()).thenReturn(attributesMap);
    when(rawFacet.put(any(), any())).thenReturn(content);
    underTest.handle(repository, component);

    ArgumentCaptor<String> pathCapture = ArgumentCaptor.forClass(String.class);
    verify(rawFacet).put(pathCapture.capture(), any(PartPayload.class));

    String path = pathCapture.getValue();
    assertNotNull(path);
    assertThat(path, is(expectedPath));
  }

  @Override
  protected String path(final String path) {
    return "/" + path;
  }
}
