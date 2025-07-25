// Modified version of your extract_probe.cpp to use the Rust face recognition library
#include "glib.h"
#include "grid.hpp"
#include "logging.h"
#include <cuda_runtime_api.h>
#include <gstnvdsmeta.h>
#include <nvbufsurface.h>
#include <opencv2/opencv.hpp>

// Include the Rust wrapper
#include "facerust_wrapper.hpp"

#define save_img TRUE
#define attach_user_meta TRUE

using namespace cv;
using namespace std;

// Struct to hold the current object data - using Rust MatchResult
struct ObjectData {
  chrono::steady_clock::time_point lastSaveTime;
  MatchResult match;  // This now uses the Rust-compatible MatchResult
  Mat imageData;
};

// Global map to store object data
map<gint, ObjectData> objectMap;

// Keep the same extract_frame_to_mat function as before
static cv::Mat extract_frame_to_mat(NvBufSurface *surface, int batch_id) {
  // ... (same implementation as your original)
}

GstPadProbeReturn extract_probe(GstPad *pad, GstPadProbeInfo *info, gpointer data) {
  CustomData *config = static_cast<CustomData *>(data);
  GstBuffer *buf = (GstBuffer *)info->data;
  NvDsBatchMeta *batch_meta = gst_buffer_get_nvds_batch_meta(buf);

  if (!batch_meta) {
    g_error("NvDsBatchMeta not found for buffer. Aborting.");
    return GST_PAD_PROBE_OK;
  }

  // Map the buffer for reading
  GstMapInfo in_map_info;
  if (!gst_buffer_map(buf, &in_map_info, GST_MAP_READ)) {
    g_error("Error: Failed to map gst buffer\n");
    return GST_PAD_PROBE_OK;
  }

  NvBufSurface *surface = (NvBufSurface *)in_map_info.data;

  // Process each frame in the batch
  for (NvDsMetaList *l_frame = batch_meta->frame_meta_list; l_frame != NULL;
       l_frame = l_frame->next) {
    NvDsFrameMeta *frame_meta = (NvDsFrameMeta *)(l_frame->data);

    // Extract frame data to OpenCV Mat
    Mat frame = extract_frame_to_mat(surface, frame_meta->batch_id);
    if (frame.empty()) {
      g_error("Error: Failed to extract frame to Mat\n");
      continue;
    }

    // Process each detected object in the frame
    gint obj_id;
    for (NvDsMetaList *l_obj = frame_meta->obj_meta_list; l_obj != NULL; l_obj = l_obj->next) {
      NvDsObjectMeta *obj_meta = (NvDsObjectMeta *)(l_obj->data);
      obj_id = obj_meta->object_id;
      
      // Same filtering logic as before
      if (strcmp(obj_meta->obj_label, "") == 0) {
        continue;
      }
      if ((config->tracker_on) && (obj_id == UNTRACKED_OBJECT_ID)) {
        DEBUG("Object is not tracked, skipping: %s", obj_meta->obj_label);
        continue;
      }

      // Extract object region from frame
      int x = (int)obj_meta->rect_params.left;
      int y = (int)obj_meta->rect_params.top;
      int width = (int)obj_meta->rect_params.width;
      int height = (int)obj_meta->rect_params.height;

      // Ensure coordinates are within frame bounds
      x = max(0, min(x, frame.cols - 1));
      y = max(0, min(y, frame.rows - 1));
      width = min(width, frame.cols - x);
      height = min(height, frame.rows - y);

      if (width <= 0 || height <= 0) {
        g_warning("Frame height or width is 0");
        continue;
      }

      // Check if 5 seconds have passed since the last save time
      auto it = objectMap.find(obj_id);
      chrono::time_point now = chrono::steady_clock::now();
      if (it == objectMap.end() ||
          chrono::duration_cast<chrono::seconds>(now - it->second.lastSaveTime).count() >
              config->face_check_interval) {

        // Extract object regions
        Rect roi(x, y, width, height);
        Mat object_img = frame(roi);

        // Update the object data in the map
        ObjectData objData;
        objData.lastSaveTime = now;
        objData.match = MatchResult("unknown", 0.0f);
        objData.match.name = obj_meta->obj_label;
        objData.imageData = object_img.clone();

        // *** KEY CHANGE: Use Rust face recognition instead of C++ ***
        try {
            // Cast to the Rust wrapper (defined in grid.hpp)
            FaceRecognitionRust* rust_face_rec = static_cast<FaceRecognitionRust*>(config->facerecognizer_rust);
            objData.match = rust_face_rec->run_one_face(object_img, 0.3f);
            DEBUG("Running Rust face recognition on %s -> %s", obj_meta->obj_label,
                  objData.match.toString().c_str());
        } catch (const std::exception& e) {
            g_warning("Rust face recognition failed: %s", e.what());
            objData.match = MatchResult("unknown", 0.0f);
        }

        // Keep the info from the last detect if the face is unknown
        if ((!objData.match.isUnknown()) || (it == objectMap.end()))
          objectMap[obj_id] = objData;
      }
      
      it = objectMap.find(obj_id);
      if (it == objectMap.end()) {
        g_error("Object not found in map");
      }

      // Change the text to name
      obj_meta->text_params.display_text = g_strdup((it->second.match.toString()).c_str());
    }
  }

  gst_buffer_unmap(buf, &in_map_info);
  return GST_PAD_PROBE_OK;
}