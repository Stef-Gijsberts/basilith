(ns org.basil.core
    (:import [java.nio ByteBuffer ByteOrder])
  (:require
   [clojure.java.process :as p]
   [clojure.math :refer [PI sin]]))

(def sample-rate-hz 44000)

(def aplay (p/start "aplay" "-f" "FLOAT_LE" "-r" (str sample-rate-hz)))

(def aplay-stdin (p/stdin aplay))

(defn float-le
  [sample]
  (let [bb (ByteBuffer/allocate 4)]
    (.order bb ByteOrder/LITTLE_ENDIAN)
    (.putFloat bb (float sample))
    (.array bb)))

(doseq [t (range 0 sample-rate-hz)]
  (.write aplay-stdin
          (float-le (sin (* 2 PI 440 (/ t sample-rate-hz))))))
