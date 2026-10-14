| Vocabulary | Constraint | Reference ms | Trie built each time ms | Prepared trie ms | Cold speedup | Prepared speedup |
|---|---|---:|---:|---:|---:|---:|
| gpt2 | three_digits | 1.305 | 7.384 | 0.259 | 0.18× | 5.04× |
| gpt2 | integer | 1.479 | 7.356 | 0.331 | 0.20× | 4.47× |
| gpt2 | identifier | 26.308 | 26.258 | 18.980 | 1.00× | 1.39× |
| gpt2 | email | 9.404 | 13.740 | 6.654 | 0.68× | 1.41× |
| gpt2 | json_record | 60.928 | 34.202 | 26.920 | 1.78× | 2.26× |
| gpt2 | free_text | 5.136 | 10.906 | 3.921 | 0.47× | 1.31× |
| qwen | three_digits | 5.650 | 28.486 | 0.399 | 0.20× | 14.15× |
| qwen | integer | 5.951 | 27.558 | 0.426 | 0.22× | 13.97× |
| qwen | identifier | 81.309 | 75.859 | 47.006 | 1.07× | 1.73× |
| qwen | email | 31.775 | 45.223 | 16.185 | 0.70× | 1.96× |
| qwen | json_record | 321.857 | 127.865 | 98.889 | 2.52× | 3.25× |
| qwen | free_text | 24.437 | 42.206 | 13.573 | 0.58× | 1.80× |
| synthetic | three_digits | 0.526 | 15.426 | 0.066 | 0.03× | 8.01× |
| synthetic | integer | 0.591 | 15.489 | 0.073 | 0.04× | 8.04× |
| synthetic | identifier | 16.391 | 32.639 | 16.508 | 0.50× | 0.99× |
| synthetic | email | 7.788 | 22.518 | 6.654 | 0.35× | 1.17× |
| synthetic | json_record | 25.233 | 19.684 | 4.028 | 1.28× | 6.26× |
| synthetic | free_text | 2.994 | 19.685 | 3.879 | 0.15× | 0.77× |
