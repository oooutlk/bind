#[cfg( test )]
mod tests {
    use bind::bind;
    use std::{
        cell::Cell,
        rc::Rc,
    };

    #[test]
    fn closure_works() {
        let i           = Rc::new( Cell::new(1) );
        let mi          = String::from("2");
        let id_id       = Rc::new( Cell::new(3) );
        let mut_id_id   = String::from("4");
        let id_expr     = "5";
        let mut_id_expr = "6";
        let e           = "7";
        let me          = "8";

        bind!(
            (   i, mut mi, ii = id_id, mut mii = mut_id_id,
                ie = id_expr.to_owned(), mut mie = mut_id_expr.to_owned(), e.to_owned(), mut me.to_owned() )
            || {
                i.set( 10 );
                mi.push( '0' );
                assert_eq!( mi, "20" );
                ii.set( 30 );
                mii.push( '0' );
                assert_eq!( mii, "40" );
                let _: Vec<String> = vec![ ie ];
                mie.push( '0' );
                assert_eq!( mie, "60" );
                let _: Vec<String> = vec![ e ];
                me.push( '0' );
                assert_eq!( me, "80" );
            }
        )();
        assert_eq!( i.get(), 10 );
        assert_eq!( mi, "2" );
        assert_eq!( id_id.get(), 30 );
        assert_eq!( mut_id_id, "4" );
    }

    #[test]
    fn move_closure_works() {
        let i           = Rc::new( Cell::new(1) );
        let mi          = String::from("2");
        let id_id       = Rc::new( Cell::new(3) );
        let mut_id_id   = String::from("4");
        let id_expr     = "5";
        let mut_id_expr = "6";
        let e           = "7";
        let me          = "8";

        bind!(
            (   i, mut mi, ii = id_id, mut mii = mut_id_id,
                ie = id_expr.to_owned(), mut mie = mut_id_expr.to_owned(), e.to_owned(), mut me.to_owned() )
            move || {
                i.set( 10 );
                mi.push( '0' );
                assert_eq!( mi, "20" );
                ii.set( 30 );
                mii.push( '0' );
                assert_eq!( mii, "40" );
                let _: Vec<String> = vec![ ie ];
                mie.push( '0' );
                assert_eq!( mie, "60" );
                let _: Vec<String> = vec![ e ];
                me.push( '0' );
                assert_eq!( me, "80" );
            }
        )();
        assert_eq!( i.get(), 10 );
        assert_eq!( mi, "2" );
        assert_eq!( id_id.get(), 30 );
        assert_eq!( mut_id_id, "4" );
    }
}
