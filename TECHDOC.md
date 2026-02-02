# LLMPROXY

This is a simple observability platform for llms.
What this does
1. Acts as a proxy between your application and any llm platform ( can be openai or self hosted ollama )
2. Lets you observe the api calls that your app is making to the ai platform
3. Relay the information back to your app from the llm platform


## Technical details

Use the rust programming language for this
1. Use 2024 edition of rust.
2. Use axum for the webserver and use reqwest for making calls.
3. Use tokio for the async runtime
4. For the database use sqlite for now using the crate `sqlx` so that we can switch to any other database later
5. For the UI use the rust framework yew.
6. In the frontend the user can see the api request and response.
7. The user can add the llm platform and the api key to make the api calls to that platform in the UI.
8. The user can generate their own api key that they use to access our proxy (this is different from the llm platform api key).
9. For the frontend authentication and login use reverse proxy auth and auto create users based on Remote-User header

user --> proxy --> llm platform
