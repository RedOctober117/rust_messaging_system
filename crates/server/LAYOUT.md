server:
    server_session:
        address,
        listener,
        users,
        router
    


server receives Message
    MessageData::User => user to users, stream to router,
    MessageData::* => forward message {
        connection = router.get(message.destination)
        send_message(connection, message)
    }