

package org.globalart.astrum.blobstore.s3;

import java.util.Arrays;
import java.util.List;

import software.amazon.awssdk.services.s3.model.ListObjectsV2Response;
import software.amazon.awssdk.services.s3.model.S3Object;

public class MockUtils
{
  public static ListObjectsV2Response getListObjectResponseForKeys(
      String... keys)
  {
    final List<S3Object> s3Objects = Arrays.stream(keys)
        .map(key -> S3Object.builder().key(key).build())
        .toList();

    return ListObjectsV2Response.builder()
        .isTruncated(false)
        .contents(s3Objects)
        .build();
  }
}
