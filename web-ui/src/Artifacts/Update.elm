module Artifacts.Update exposing (..)

import Dict
import Navigation
import Models exposing (..)
import Messages
    exposing
        ( createUrl
        , AppMsg(AppError)
        , Route(ArtifactNameRoute, ArtifactCreateRoute)
        )
import Utils exposing (assertOr)
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models exposing (..)
import Artifacts.Commands exposing (updateArtifacts, createArtifacts)


update : Msg -> Model -> ( Model, Cmd AppMsg )
update msg model =
    case msg of
        ReceivedArtifacts artifactList ->
            handleReceived model artifactList

        ShowArtifacts ->
            ( model, Navigation.newUrl artifactsUrl )

        ShowArtifact name ->
            ( model
            , Navigation.newUrl <|
                artifactNameUrl (indexNameUnchecked name)
            )

        CreateArtifact ->
            ( model
            , Navigation.newUrl <| "#" ++ createUrl
            )

        ChangeColumns columns ->
            let
                s =
                    model.state

                state =
                    { s | columns = columns }
            in
                ( { model | state = state }, Cmd.none )

        ChangeTextViewState textView ->
            let
                s =
                    model.state

                state =
                    { s | textView = textView }
            in
                ( { model | state = state }, Cmd.none )

        ChangeSearch search ->
            let
                s =
                    model.state

                state =
                    { s | search = search }
            in
                ( { model | state = state }, Cmd.none )

        EditArtifact option ->
            case option of
                ChangeChoice artifact edited ->
                    let
                        -- update revision so that any warnings of prior
                        -- change go away
                        e =
                            { edited | revision = artifact.revision }

                        artifacts =
                            setEdited model.artifacts artifact (Just e)
                    in
                        ( { model | artifacts = artifacts }
                        , Cmd.none
                        )

                CreateChoice edited ->
                    ( { model | create = Just edited }, Cmd.none )

        CancelEditArtifact option ->
            case option of
                ChangeChoice artifact _ ->
                    ( { model | artifacts = setEdited model.artifacts artifact Nothing }
                    , Cmd.none
                    )

                CreateChoice _ ->
                    ( { model | create = Nothing }, Cmd.none )

        SaveArtifact option ->
            case option of
                ChangeChoice artifact edited ->
                    let
                        model2 =
                            log model <| "trying to save " ++ (toString artifact.name.value)

                        model3 =
                            { model2 | jsonId = model.jsonId + 1 }

                        value =
                            Dict.singleton artifact.id (getEditable artifact)
                    in
                        ( model3, updateArtifacts model value )

                CreateChoice edited ->
                    let
                        model2 =
                            { model | jsonId = model.jsonId + 1 }
                    in
                        ( model2, createArtifacts model [ edited ] )


{-| set the edited variable on the requested artifact
-}
setEdited : Artifacts -> Artifact -> Maybe EditableArtifact -> Artifacts
setEdited artifacts art edited =
    Dict.insert art.id { art | edited = edited } artifacts


{-| we need to make sure we keep any edited data that has not been applied
-}
handleReceived : Model -> List Artifact -> ( Model, Cmd AppMsg )
handleReceived model artifactList =
    let
        processed =
            List.map (processNew model) artifactList

        artifacts =
            artifactsFromList <| List.map (\p -> p.artifact) processed

        names =
            nameIds artifacts

        routes =
            List.filterMap (\p -> p.route) processed

        clear_create =
            List.map (\p -> p.clear_create) processed
                |> List.any (\a -> a)

        logs =
            let
                l =
                    List.filterMap (\p -> p.log) processed
            in
                { all = List.append l model.logs.all }

        _ =
            assertOr ((List.length routes) <= 1) 0 "impossible routes"

        ( route, cmd ) =
            case List.head routes of
                Just r ->
                    ( ArtifactNameRoute r
                    , Navigation.newUrl <| artifactNameUrl r
                    )

                Nothing ->
                    ( model.route, Cmd.none )

        create =
            if clear_create then
                Nothing
            else
                model.create

        new_model =
            { model
                | artifacts = artifacts
                , names = names
                , route = route
                , create = create
                , logs = logs
            }
    in
        ( new_model, cmd )


{-| get the edited, keeping in mind that changes may have been applied
-}
handleEditedReceived : Artifact -> Artifact -> Maybe EditableArtifact
handleEditedReceived oldArt newArt =
    if oldArt.revision == newArt.revision then
        oldArt.edited
    else
        case oldArt.edited of
            Just e ->
                if editedEqual e <| createEditable newArt then
                    -- the changes were applied
                    Nothing
                else
                    -- The changes have not been applied
                    -- but the artifact has changed (by someone else)!
                    -- That's fine, keep the old edited data
                    -- and edited.revision will be used for a warning
                    Just e

            Nothing ->
                Nothing


processNew :
    Model
    -> Artifact
    ->
        { log : Maybe String
        , clear_create : Bool
        , route : Maybe String
        , artifact : Artifact
        }
processNew model newArt =
    case Dict.get newArt.id model.artifacts of
        Just oldArt ->
            -- artifact exists and is being updated
            let
                edited =
                    handleEditedReceived oldArt newArt

                route =
                    case model.route of
                        ArtifactNameRoute route_name ->
                            if (indexNameUnchecked route_name) == oldArt.name.value then
                                -- Artifact we are viewing got changed. Always go to
                                -- the new name
                                Just newArt.name.value
                            else
                                Nothing

                        _ ->
                            Nothing
            in
                { clear_create = False
                , route = route
                , artifact = { newArt | edited = edited }
                , log = Nothing
                }

        Nothing ->
            -- artifact is new
            let
                edited =
                    createEditable newArt

                ( log, clear ) =
                    case model.create of
                        Just e ->
                            if editedEqual e edited then
                                ( Just "Creation Successful", True )
                            else
                                ( Nothing, False )

                        _ ->
                            ( Nothing, False )
            in
                { clear_create = clear
                , route = Nothing
                , artifact = newArt
                , log = log
                }
