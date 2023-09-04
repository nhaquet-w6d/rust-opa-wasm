#  Copyright 2022 The Matrix.org Foundation C.I.C.
# 
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
# 
#      http://www.apache.org/licenses/LICENSE-2.0
# 
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.

package test

response_simple := http.send({
		"method" : "GET",
		"url": "https://google.com"
})

response_cache := http.send({
		"method" : "GET",
		"url": "https://google.com",
		"cahce": "true"
})

response_force_decode_yaml := http.send({
		"method" : "GET",
		"url": "https://google.com",
		"force_yaml_decode": true
})

response_force_decode_json := http.send({
		"method" : "GET",
		"url": "https://google.com",
		"force_yaml_decode": true
})

response_redirect := http.send({
		"method" : "GET",
		"url": "https://google.com",
		"enable_redirect": true
})


response_max_retry := http.send({
		"method" : "GET",
		"url": "https://google.com",
		"max_retry_atempts": 10
})
