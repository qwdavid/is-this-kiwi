import sys
import os
# TensorFlow のログ出力を抑える（0 = 全部、3 = エラーのみ）
os.environ.setdefault('TF_CPP_MIN_LOG_LEVEL', '3')
import tensorflow as tf
from tensorflow import keras
import numpy as np

class KiwiAI:
    CATEGORIES = ["A","F"]
    CATEGORIES_NAME = ["キーウィ","キウイ"]
    IMG_SIZE = 150
    INPUT_SHAPE = (IMG_SIZE, IMG_SIZE, 3)

    def __init__(self, root_dir="./model"):
        self.MODEL_ROOT_DIR = root_dir
        self.MODEL_PATH = os.path.join(self.MODEL_ROOT_DIR, 'model_predict.json')
        self.WEIGHT_PATH = os.path.join(self.MODEL_ROOT_DIR, 'model_predict.weights.h5')
        self.model = None
        self.img_path = None

    def loadModel(self):
        with open(self.MODEL_PATH) as json_file:
            json_string = json_file.read()
        self.model = keras.models.model_from_json(json_string)
        self.model.load_weights(self.WEIGHT_PATH)

    def imageLoader(self, image_path: str):
        self.img_path = image_path

    def outputProcess(self):
        img = keras.preprocessing.image.load_img(
            self.img_path,
            target_size=self.INPUT_SHAPE[:2]
        )
        x = keras.preprocessing.image.img_to_array(img)
        x = np.expand_dims(x, axis=0)

        features = self.model.predict(x, verbose=0)
        idx = np.argmax(features[0])
        return self.CATEGORIES_NAME[idx]

    def run(self, image_path: str):
        self.loadModel()
        self.imageLoader(image_path)
        return self.outputProcess()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("ERROR: image path not provided")
        sys.exit(1)

    image_path = sys.argv[1]
    ai = KiwiAI()
    result = ai.run(image_path)

    # 判定結果はこの標準出力を Rust 側が読む
    print(result)
