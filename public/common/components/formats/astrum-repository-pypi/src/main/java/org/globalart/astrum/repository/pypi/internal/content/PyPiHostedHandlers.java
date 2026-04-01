
package org.globalart.astrum.repository.pypi.internal.content;

import java.io.IOException;
import java.util.LinkedHashMap;
import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.pypi.content.PypiContentFacet;
import org.globalart.astrum.repository.pypi.internal.SignablePyPiPackage;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;
import org.globalart.astrum.repository.view.payloads.TempBlobPartPayload;

import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.ACTION_FILE_UPLOAD;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.FIELD_ACTION;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.FIELD_CONTENT;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.GPG_SIGNATURE;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.name;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.path;
import static org.globalart.astrum.repository.pypi.internal.PyPiStorageUtils.addAttribute;

@Component
public class PyPiHostedHandlers
{
  public final Handler getPackage = context -> {
    State state = context.getAttributes().require(TokenMatcher.State.class);
    Content content = context.getRepository().facet(PyPiHostedFacet.class).getPackage(path(state));
    return content != null ? HttpResponses.ok(content) : HttpResponses.notFound();
  };

  public final Handler getRootIndex = context -> {
    Content content = context.getRepository().facet(PyPiHostedFacet.class).getRootIndex();
    return content != null ? HttpResponses.ok(content) : HttpResponses.notFound();
  };

  public final Handler getIndex = context -> {
    State state = context.getAttributes().require(TokenMatcher.State.class);
    Content content = context.getRepository().facet(PyPiHostedFacet.class).getIndex(name(state));
    return content != null ? HttpResponses.ok(content) : HttpResponses.notFound();
  };

  public final Handler postContent = context -> {
    SignablePyPiPackage pyPiPackage;
    try {
      pyPiPackage = extractPayloads(context);
    }
    catch (IllegalArgumentException e) {
      return HttpResponses.badRequest(e.getMessage());
    }
    String action = pyPiPackage.getAttributes().get(FIELD_ACTION);
    if (!ACTION_FILE_UPLOAD.equals(action)) {
      pyPiPackage.getWheelPayload().close();
      if (pyPiPackage.getGpgSignature() != null) {
        pyPiPackage.getGpgSignature().close();
      }
      return HttpResponses.badRequest("Unsupported :action, found: " + action);
    }
    context.getRepository().facet(PyPiHostedFacet.class).upload(pyPiPackage);
    return HttpResponses.ok();
  };

  private SignablePyPiPackage extractPayloads(final Context context) throws IOException {
    checkNotNull(context);
    Map<String, String> attributes = new LinkedHashMap<>();
    Request request = context.getRequest();
    TempBlobPartPayload contentBlob = null;
    TempBlobPartPayload signatureBlob = null;

    if (!request.isMultipart()) {
      throw new IllegalArgumentException("Expected a multipart upload request");
    }

    for (PartPayload payload : checkNotNull(request.getMultiparts())) {
      if (payload.isFormField()) {
        addAttribute(attributes, payload);
      }
      else if (FIELD_CONTENT.equals(payload.getFieldName())) {
        if (contentBlob != null) {
          throw new IllegalArgumentException("Multiple content parts are not supported");
        }
        contentBlob = createBlobFromPayload(payload, context.getRepository());
      }
      else if (GPG_SIGNATURE.equals(payload.getFieldName())) {
        if (signatureBlob != null) {
          throw new IllegalArgumentException("Multiple signature parts are not supported");
        }
        signatureBlob = createBlobFromPayload(payload, context.getRepository());
      }
    }
    if (contentBlob == null) {
      throw new IllegalArgumentException("Missing content part");
    }
    return new SignablePyPiPackage(contentBlob, attributes, signatureBlob);
  }

  private TempBlobPartPayload createBlobFromPayload(final PartPayload payload, final Repository repository)
      throws IOException
  {
    PypiContentFacet contentFacet = repository.facet(PypiContentFacet.class);
    return new TempBlobPartPayload(payload, contentFacet.getTempBlob(payload));
  }
}
