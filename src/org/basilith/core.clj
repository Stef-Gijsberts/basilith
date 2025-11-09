(ns org.basilith.core
  (:import [java.nio ByteBuffer ByteOrder])
  (:require
   [clojure.java.process :as p]
   [clojure.math :refer [PI sin]]))

(def sample-rate-hz 44000)
(def chunk-size (/ sample-rate-hz 16))

(def aplay (p/start "aplay" "--buffer-size" (str chunk-size) "-f" "FLOAT_LE" "-r" (str sample-rate-hz)))

(def aplay-stdin (p/stdin aplay))

(def settings (atom {:frequency-hz 440
                     :gain-factor 1.0}))

(comment
  (swap! settings assoc :frequency-hz (+ 400 (rand-int 200)))
  (swap! settings assoc :gain-factor (rand)))

(def buffer (ByteBuffer/allocate (* 4 chunk-size)))
(.order buffer ByteOrder/LITTLE_ENDIAN)

(def audio
  (future
    (loop [chunk-index 0]
      (let [s @settings]
        (.clear buffer)
        (dotimes [i chunk-size]
          (let [sample-index (+ (* chunk-index chunk-size) i)
                t-seconds (/ sample-index sample-rate-hz)]
            (.putFloat buffer (* (:gain-factor s) (sin (* 2.0 PI (:frequency-hz s) t-seconds))))))
        (.write aplay-stdin (.array buffer))
        (when-not (Thread/interrupted)
          (recur (inc chunk-index)))))))

(comment
  (future-cancel audio))